#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
    System::Threading::CreateMutexW,
    UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE},
};

const INSTANCE_NAME: &str = "Local\\MistriaTracker.SingleInstance";
const WINDOW_TITLE: &str = "Mistria Tracker";

/// OS-backed guard retained for the entire lifetime of the first desktop process.
#[cfg(windows)]
pub struct InstanceGuard(HANDLE);

#[cfg(windows)]
impl InstanceGuard {
    pub fn acquire() -> Option<Self> {
        Self::acquire_named(INSTANCE_NAME)
    }

    fn acquire_named(name: &str) -> Option<Self> {
        let name = wide(name);
        // CreateMutexW creates the mutex on first use and returns a handle when
        // another tracker already owns it; that handle is closed immediately.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return None;
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return None;
        }
        Some(Self(handle))
    }
}

#[cfg(windows)]
impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

#[cfg(windows)]
pub fn focus_existing_tracker() {
    let title = wide(WINDOW_TITLE);
    let window = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if !window.is_null() {
        unsafe {
            ShowWindow(window, SW_RESTORE);
            SetForegroundWindow(window);
        }
    }
}

#[cfg(not(windows))]
pub struct InstanceGuard;

#[cfg(not(windows))]
impl InstanceGuard {
    pub fn acquire() -> Option<Self> {
        Some(Self)
    }
}

#[cfg(not(windows))]
pub fn focus_existing_tracker() {}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(all(test, windows))]
mod tests {
    use super::InstanceGuard;

    #[test]
    fn the_second_tracker_instance_is_rejected_while_the_first_holds_the_named_mutex() {
        let name = format!("MistriaTracker-test-{}", std::process::id());
        let first = InstanceGuard::acquire_named(&name).unwrap();

        assert!(InstanceGuard::acquire_named(&name).is_none());

        drop(first);
    }
}
