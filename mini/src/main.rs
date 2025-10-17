use anyhow::Context;
use std::{env, fs, path::PathBuf};

use mini::{ast::Program, codegen::{Codegen, host_triple}, link::link_exe, parser::Parser};
use inkwell::context::Context as LlvmContext;

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: mini <input.mini> <output-exe>");
        std::process::exit(1);
    }
    let input = PathBuf::from(&args[0]);
    let out_exe = PathBuf::from(&args[1]);

    let src = fs::read_to_string(&input).with_context(|| format!("reading {:?}", input))?;
    let program: Program = Parser::parse(&src)?;

    let ctx = LlvmContext::create();
    let triple = host_triple();
    let mut cg = Codegen::new(&ctx, &triple);
    cg.emit_program(&program)?;
    let obj = out_exe.with_extension("o");
    cg.write_object(&triple, &obj)?;
    link_exe(&obj, &out_exe)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&out_exe)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&out_exe, perm)?;
    }

    println!("Built {}", out_exe.display());
    Ok(())
}
