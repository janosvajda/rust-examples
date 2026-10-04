//! Platform-specific linking helpers for turning object files into executables.

use anyhow::{bail, Result};

/// Invoke the appropriate system linker to produce a runnable binary.
// Each platform block below ends with `return Ok(())`. Only one block is compiled
// on any given OS, so clippy sees that `return` as the last statement and calls it
// needless. The `return`s keep every block correct on its own, whichever OS it runs on.
#[allow(clippy::needless_return)]
pub fn link_exe(obj: &std::path::Path, out_exe: &std::path::Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let sdk = String::from_utf8(
            Command::new("xcrun")
                .args(["--sdk", "macosx", "--show-sdk-path"])
                .output()?
                .stdout,
        )?
        .trim()
        .to_string();

        // Detect host macOS version (major.minor) and use for both min & current
        let prod = String::from_utf8(
            Command::new("sw_vers").args(["-productVersion"]).output()?.stdout,
        )?;
        let ver = prod.trim();
        let mut it = ver.split('.');
        let major = it.next().unwrap_or("13");
        let minor = it.next().unwrap_or("0");
        let platform_ver = format!("{}.{}", major, minor);

        let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };

        let status = Command::new("ld")
            .args([
                "-o",
                out_exe.to_str().unwrap(),
                "-arch",
                arch,
                // supply minimum and current macOS platform versions to satisfy ld
                "-platform_version",
                "macos",
                &platform_ver,
                &platform_ver,
                "-syslibroot",
                &sdk,
                "-e",
                "_main",
                obj.to_str().unwrap(),
                "-lSystem",
            ])
            .status()?;
        if !status.success() {
            bail!("ld failed");
        }
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        // Prefer gcc when available for convenience; otherwise fall back to plain `ld`.
        if std::process::Command::new("gcc").arg("--version").output().is_ok() {
            let status = std::process::Command::new("gcc")
                .args([obj.to_str().unwrap(), "-o", out_exe.to_str().unwrap(), "-lc"])
                .status()?;
            if !status.success() {
                bail!("gcc link failed");
            }
        } else {
            let status = std::process::Command::new("ld")
                .args([obj.to_str().unwrap(), "-o", out_exe.to_str().unwrap(), "-lc"])
                .status()?;
            if !status.success() {
                bail!("ld failed");
            }
        }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        // Use MSVC's linker directly; rely on the CRT and legacy printf symbols.
        let status = std::process::Command::new("link.exe")
            .args([
                obj.to_str().unwrap(),
                &format!("/OUT:{}", out_exe.to_str().unwrap()),
                "msvcrt.lib",
                "legacy_stdio_definitions.lib",
            ])
            .status()?;
        if !status.success() {
            bail!("link.exe failed");
        }
        return Ok(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    compile_error!("Unsupported OS: this compiler currently supports macOS, Linux, and Windows.");
}
