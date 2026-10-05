//! Platform-specific linking helpers for turning object files into executables.

use anyhow::{bail, Context, Result};
use std::{path::Path, process::Command};

/// Ask the C compiler driver to select the linker, startup code and C runtime.
/// `cc` must be installed and configured for the same host target as LLVM.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn link_exe(obj: &Path, out_exe: &Path) -> Result<()> {
    let status = Command::new("cc")
        .arg(obj)
        .arg("-o")
        .arg(out_exe)
        .status()
        .context(
            "could not run `cc`: install a C compiler and its platform SDK/development libraries",
        )?;
    if !status.success() {
        bail!("`cc` failed to link {} ({status})", out_exe.display());
    }
    Ok(())
}

/// Use the MSVC linker from a Visual Studio developer shell.
#[cfg(target_os = "windows")]
pub fn link_exe(obj: &Path, out_exe: &Path) -> Result<()> {
    let mut output_arg = std::ffi::OsString::from("/OUT:");
    output_arg.push(out_exe);
    let status = Command::new("link.exe")
        .arg(obj)
        .arg(output_arg)
        .args(["msvcrt.lib", "legacy_stdio_definitions.lib"])
        .status()
        .context("could not run `link.exe`: use a Visual Studio developer shell")?;
    if !status.success() {
        bail!("`link.exe` failed to link {} ({status})", out_exe.display());
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
compile_error!("Unsupported OS: this compiler currently supports macOS, Linux, and Windows.");
