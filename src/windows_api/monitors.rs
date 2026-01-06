//! Monitor enumeration using Windows API

use crate::types::{MonitorInfo, WindowRect};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::Mutex;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

/// Enumerate all connected monitors
pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let monitors: Mutex<Vec<(HMONITOR, usize)>> = Mutex::new(Vec::new());

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitors_callback),
            LPARAM(&monitors as *const _ as isize),
        );
    }

    let handles = monitors.into_inner().unwrap_or_default();

    handles
        .into_iter()
        .filter_map(|(handle, index)| get_monitor_info(handle, index))
        .collect()
}

unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &*(lparam.0 as *const Mutex<Vec<(HMONITOR, usize)>>);

    if let Ok(mut guard) = monitors.lock() {
        let index = guard.len();
        guard.push((hmonitor, index));
    }

    TRUE
}

/// Get detailed information about a monitor
fn get_monitor_info(handle: HMONITOR, index: usize) -> Option<MonitorInfo> {
    unsafe {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        if !GetMonitorInfoW(handle, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
            return None;
        }

        let device_name = OsString::from_wide(
            &info.szDevice[..info
                .szDevice
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(info.szDevice.len())],
        )
        .to_string_lossy()
        .to_string();

        let is_primary = info.monitorInfo.dwFlags & 1 != 0; // MONITORINFOF_PRIMARY

        let bounds = WindowRect {
            left: info.monitorInfo.rcMonitor.left,
            top: info.monitorInfo.rcMonitor.top,
            right: info.monitorInfo.rcMonitor.right,
            bottom: info.monitorInfo.rcMonitor.bottom,
        };

        let work_area = WindowRect {
            left: info.monitorInfo.rcWork.left,
            top: info.monitorInfo.rcWork.top,
            right: info.monitorInfo.rcWork.right,
            bottom: info.monitorInfo.rcWork.bottom,
        };

        // Create a friendly name
        let name = if is_primary {
            format!("Display {} (Primary)", index + 1)
        } else {
            format!("Display {}", index + 1)
        };

        Some(MonitorInfo {
            handle: handle.0 as isize,
            name,
            device_name,
            bounds,
            work_area,
            is_primary,
            display_index: index,
        })
    }
}
