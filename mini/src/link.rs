use anyhow::{bail, Result};

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
        if which::which("gcc").is_ok() {
            let status = std::process::Command::new("gcc")
                .args([obj.to_str().unwrap(), "-o", out_exe.to_str().unwrap(), "-lc"])
                .status()?;
            if !status.success() {
                bail!("gcc link failed");
            }
        } else {
            let linker = which::which("ld.lld")
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "ld".into());
            let status = std::process::Command::new(&linker)
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
