use crate::Result;
use anyhow::Context as _;
use anyhow::bail;
use std::path::Path;
#[cfg(not(target_os = "macos"))]
use std::process::Command;

#[cfg(target_os = "macos")]
pub(crate) fn filesystem_type(path: &Path) -> Result<String> {
    let stat = nix::sys::statfs::statfs(path)
        .with_context(|| format!("Failed to determine filesystem for {}", path.display()))?;
    Ok(stat.filesystem_type_name().to_owned())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn filesystem_type(path: &Path) -> Result<String> {
    let output = Command::new("findmnt")
        .args([
            "--first-only",
            "--noheadings",
            "--output",
            "FSTYPE",
            "--target",
        ])
        .arg(path)
        .output()
        .context("Failed to run findmnt")?;
    if !output.status.success() {
        bail!(
            "Failed to determine filesystem for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(crate) fn check_system_settings() -> Result {
    let policies = match std::fs::read_dir("/sys/devices/system/cpu/cpufreq") {
        Ok(policies) => policies,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("Failed to read CPU frequency policies"),
    };

    for policy in policies {
        let policy = policy?;
        if !policy.file_name().as_encoded_bytes().starts_with(b"policy") {
            continue;
        }
        let path = policy.path();
        let min = read_setting(&path.join("scaling_min_freq"))?;
        let max = read_setting(&path.join("scaling_max_freq"))?;
        if min != max {
            bail!(
                "CPU frequency limits differ for {} ({min}–{max} kHz). Set equal limits with:\n\
                sudo cpupower frequency-set --min <frequency> --max <frequency>",
                path.display()
            );
        }
    }

    let boost = Path::new("/sys/devices/system/cpu/cpufreq/boost");
    let no_turbo = Path::new("/sys/devices/system/cpu/intel_pstate/no_turbo");
    let boost_enabled = if boost.try_exists()? {
        read_setting(boost)? != 0
    } else if no_turbo.try_exists()? {
        read_setting(no_turbo)? == 0
    } else {
        false
    };
    if boost_enabled {
        bail!(
            "CPU boost is enabled, please run:\n\
            sudo cpupower set --boost 0"
        );
    }

    Ok(())
}

fn read_setting(path: &Path) -> Result<u64> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?
        .trim()
        .parse()
        .with_context(|| format!("Invalid CPU setting in {}", path.display()))
}
