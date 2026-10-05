//! Command-line driver: parse source, write LLVM IR, compile it with the
//! installed LLVM, and link the result into a native executable.

use anyhow::Context;
use std::{env, fs, path::PathBuf};

use mini::{codegen, link::link_exe, llvm, parser::Parser};

fn main() -> anyhow::Result<()> {
    // CLI expects `<input.mini> <output-exe>` for simplicity.
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: mini <input.mini> <output-exe>");
        std::process::exit(1);
    }
    let input = PathBuf::from(&args[0]);
    let out_exe = PathBuf::from(&args[1]);

    // 1. Check that an LLVM 16+ is installed before doing any work.
    let llc = llvm::llc_program();
    llvm::check_version(&llc)?;

    // 2. Source text → syntax tree → LLVM IR text, saved next to the executable.
    let src = fs::read_to_string(&input).with_context(|| format!("reading {:?}", input))?;
    let program = Parser::parse(&src)?;
    let ir = codegen::generate(&program)?;
    let ll_file = out_exe.with_extension("ll");
    fs::write(&ll_file, ir).with_context(|| format!("writing {:?}", ll_file))?;

    // 3. LLVM IR → machine code (object file) → executable.
    let obj = out_exe.with_extension("o");
    llvm::compile_to_object(&llc, &ll_file, &obj)?;
    link_exe(&obj, &out_exe)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&out_exe)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&out_exe, perm)?;
    }

    // Basic success message so users know where the binary landed.
    println!(
        "Built {} (LLVM IR: {})",
        out_exe.display(),
        ll_file.display()
    );
    Ok(())
}
