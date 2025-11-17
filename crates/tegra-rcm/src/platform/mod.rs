mod linux;

use super::error::Result;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "macos")] {
        // nothing yet
    } else if #[cfg(target_os = "windows")] {
        // nothing yet
    } else {
        compile_error!("Unsupported OS");
    }
}

/// Validates the current environment to if the exploit can be used
///
/// This should be called before trying to connect to a Switch
pub fn validate_environment() -> Result<()> {
    #[cfg(target_os = "linux")]
    linux::validate_environment();
    Ok(())
}
