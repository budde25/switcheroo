use crate::Result;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_os = "macos", target_os = "linux"))] {
        mod unix;
        pub use unix::SwitchDevice;
        pub use unix::SwitchHandle;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::SwitchDevice;
    } else {
        compile_error!("Unsupported OS");
    }
}

pub(crate) const RCM_VID: u16 = 0x0955;
pub(crate) const RCM_PID: u16 = 0x7321;

pub(crate) trait Device {
    fn find_device() -> Result<SwitchDevice>;
    fn init(&mut self) -> Result<SwitchHandle>;
}

pub(crate) trait DeviceHandle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
}
