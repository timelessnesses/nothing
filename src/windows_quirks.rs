#[cfg(target_os = "windows")]
use winapi;

pub fn windows_hide_console() {
    #[cfg(target_os = "windows")]
    fn hide() {
        unsafe {
            let cw = winapi::um::wincon::GetConsoleWindow();
            if cw != std::ptr::null_mut() {
                winapi::um::winuser::ShowWindow(cw, winapi::um::winuser::SW_HIDE);
            }
        }
    }

    #[cfg(target_os = "windows")]
    hide()
}
