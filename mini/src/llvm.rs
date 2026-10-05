//! Talking to the installed LLVM: check its version and run `llc`.
//!
//! Mini does not build LLVM into itself. It writes LLVM IR as plain text and
//! asks the `llc` program of whatever LLVM is installed to turn that text into
//! an object file of real machine code.

use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// The oldest LLVM whose IR text Mini's code generator writes.
pub const MIN_LLVM_VERSION: u32 = 16;

/// The `llc` program to use: `llc` from the PATH, or the one named in `MINI_LLC`
/// (for example `MINI_LLC=llc-18` on Ubuntu, where the tools carry a version suffix).
pub fn llc_program() -> String {
    std::env::var("MINI_LLC").unwrap_or_else(|_| "llc".to_string())
}

/// Run `llc --version` and make sure it is LLVM 16 or newer. Returns the major version.
pub fn check_version(llc: &str) -> Result<u32> {
    let output = Command::new(llc).arg("--version").output().with_context(|| {
        format!("could not run `{llc}`: Mini needs LLVM {MIN_LLVM_VERSION} or newer, with `llc` on your PATH")
    })?;
    if !output.status.success() {
        bail!(
            "`{llc} --version` failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let major = major_version(&text)
        .ok_or_else(|| anyhow!("could not read the LLVM version from `{llc} --version`"))?;
    if major < MIN_LLVM_VERSION {
        bail!("Mini needs LLVM {MIN_LLVM_VERSION} or newer, but `{llc}` is LLVM {major}");
    }
    Ok(major)
}

/// Find the major version in text like "Homebrew LLVM version 21.1.3".
fn major_version(text: &str) -> Option<u32> {
    let after = text.split("LLVM version ").nth(1)?;
    after.split('.').next()?.trim().parse().ok()
}

/// Compile an LLVM IR text file (`.ll`) into an object file (`.o`) for this machine.
pub fn compile_to_object(llc: &str, ll_file: &Path, obj_file: &Path) -> Result<()> {
    let status = Command::new(llc)
        .arg("-filetype=obj") // write machine code, not assembly text
        .arg("-relocation-model=pic") // position-independent code, which modern linkers expect
        .arg(ll_file)
        .arg("-o")
        .arg(obj_file)
        .status()
        .with_context(|| format!("could not run `{llc}`"))?;
    if !status.success() {
        bail!("`{llc}` failed to compile {}", ll_file.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::major_version;

    #[test]
    fn reads_the_major_version() {
        assert_eq!(
            major_version("Homebrew LLVM version 21.1.3\n  Optimized build."),
            Some(21)
        );
        assert_eq!(
            major_version("LLVM (http://llvm.org/):\n  Ubuntu LLVM version 18.1.3"),
            Some(18)
        );
        assert_eq!(major_version("something else"), None);
    }
}
