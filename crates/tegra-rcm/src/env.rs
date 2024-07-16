/// checks to see if there is a misconfiguration in the environment, this can be called at any time
use crate::error::Result;

/// Validates the environment
#[cfg(target_os = "linux")]
pub fn check_env() -> Result<()> {
    use log::info;

    const UDEV_RULES: &str = "/etc/udev/rules.d/99-switch.rules";
    const FLATPAK_UDEV_RULES: &str = "/run/host/etc/udev/rules.d/99-switch.rules";

    if std::env::var("SWITCHEROO_SKIP_UDEV_CHECK").is_ok() {
        return Ok(());
    }

    let path = match std::env::var("FLATPAK_ID") {
        Ok(_) => FLATPAK_UDEV_RULES,
        Err(_) => UDEV_RULES,
    };

    let path = std::path::Path::new(path);
    info!("checking for udev rules at {}", path.display());
    if path.exists() {
        Ok(())
    } else {
        Err(crate::SwitchError::UdevRulesNotFound)
    }
}

/// Validates the environment
#[cfg(not(target_os = "linux"))]
pub fn check_env() -> Result<()> {
    Ok(())
}
