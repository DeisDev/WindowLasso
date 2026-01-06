//! Window enumeration and manipulation using Windows API

use crate::types::{MonitorInfo, WindowInfo, WindowRect};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::Mutex;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassLongPtrW, GetIconInfo, GetWindowLongPtrW, GetWindowPlacement,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, SendMessageTimeoutW, SetWindowPos, ShowWindow, GCLP_HICON, GCLP_HICONSM,
    GWL_EXSTYLE, GWL_STYLE, HWND_TOP, ICONINFO, SMTO_ABORTIFHUNG, SWP_NOZORDER, SWP_SHOWWINDOW,
    SW_MAXIMIZE, SW_RESTORE, WINDOWPLACEMENT, WM_GETICON, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_VISIBLE,
};

/// Enumerate all visible application windows
pub fn enumerate_windows(monitors: &[MonitorInfo]) -> Vec<WindowInfo> {
    let windows: Mutex<Vec<WindowInfo>> = Mutex::new(Vec::new());
    let monitors_clone = monitors.to_vec();

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&windows as *const _ as isize),
        );
    }

    let mut result = windows.into_inner().unwrap_or_default();

    // Update off-screen status based on monitors
    for window in &mut result {
        window.is_offscreen = !is_window_on_any_monitor(&window.rect, &monitors_clone);
        window.monitor_name = find_window_monitor(&window.rect, &monitors_clone);
    }

    // Sort: off-screen windows first, then by title
    result.sort_by(|a, b| match (a.is_offscreen, b.is_offscreen) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
    });

    result
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &*(lparam.0 as *const Mutex<Vec<WindowInfo>>);

    // Check if window is minimized
    let is_minimized = IsIconic(hwnd).as_bool();

    // For non-minimized windows, check visibility
    if !is_minimized && !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    // Get window style
    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

    // For non-minimized windows, require WS_VISIBLE
    // Minimized windows may not have WS_VISIBLE set but we still want them
    if !is_minimized && style & WS_VISIBLE.0 == 0 {
        return TRUE;
    }

    // Skip tool windows and other non-app windows
    if ex_style & WS_EX_TOOLWINDOW.0 != 0 {
        return TRUE;
    }

    // Skip windows with WS_EX_NOACTIVATE unless they have WS_EX_APPWINDOW
    if ex_style & WS_EX_NOACTIVATE.0 != 0 && ex_style & WS_EX_APPWINDOW.0 == 0 {
        return TRUE;
    }

    // Get window title
    let title_len = GetWindowTextLengthW(hwnd);
    if title_len == 0 {
        return TRUE;
    }

    let mut title_buffer: Vec<u16> = vec![0; (title_len + 1) as usize];
    let actual_len = GetWindowTextW(hwnd, &mut title_buffer);
    if actual_len == 0 {
        return TRUE;
    }

    let title = OsString::from_wide(&title_buffer[..actual_len as usize])
        .to_string_lossy()
        .to_string();

    // Skip empty titles and some system windows
    if title.is_empty() || should_skip_window(&title) {
        return TRUE;
    }

    // Get window rect - for minimized windows, use the restored rect from placement
    let rect = if is_minimized {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        if GetWindowPlacement(hwnd, &mut placement).is_err() {
            return TRUE;
        }
        placement.rcNormalPosition
    } else {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_err() {
            return TRUE;
        }
        r
    };

    // Skip zero-size windows (but be lenient for minimized windows)
    if !is_minimized && (rect.right - rect.left <= 0 || rect.bottom - rect.top <= 0) {
        return TRUE;
    }

    // Get process information
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    let process_name = get_process_name(process_id).unwrap_or_else(|| "Unknown".to_string());

    // Skip certain processes
    if should_skip_process(&process_name) {
        return TRUE;
    }

    // Extract window/process icon
    let (icon_rgba, icon_size) = get_window_icon(hwnd, process_id);

    let window_info = WindowInfo {
        hwnd: hwnd.0 as isize,
        title,
        process_name,
        process_id,
        rect: WindowRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        },
        is_visible: !is_minimized,
        is_offscreen: false,
        is_minimized,
        monitor_name: None,
        icon_rgba,
        icon_size,
    };

    if let Ok(mut guard) = windows.lock() {
        guard.push(window_info);
    }

    TRUE
}

/// Get the process name from a process ID
fn get_process_name(process_id: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        )
        .ok()?;

        let mut name_buffer: Vec<u16> = vec![0; 260];
        let len = GetModuleBaseNameW(handle, None, &mut name_buffer);

        if len > 0 {
            let name = OsString::from_wide(&name_buffer[..len as usize])
                .to_string_lossy()
                .to_string();
            // Remove .exe extension if present
            Some(name.trim_end_matches(".exe").to_string())
        } else {
            None
        }
    }
}

/// Get window icon as RGBA pixel data
fn get_window_icon(hwnd: HWND, _process_id: u32) -> (Option<Vec<u8>>, u32) {
    const ICON_SIZE: u32 = 32;

    unsafe {
        // Try multiple methods to get the icon
        let hicon = get_icon_from_window(hwnd);

        if let Some(icon) = hicon {
            if let Some(rgba) = icon_to_rgba(icon, ICON_SIZE) {
                // Don't destroy icons obtained from GetClassLongPtr
                return (Some(rgba), ICON_SIZE);
            }
        }

        (None, ICON_SIZE)
    }
}

/// Try to get icon handle from a window using various methods
unsafe fn get_icon_from_window(hwnd: HWND) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON>
{
    use windows::Win32::UI::WindowsAndMessaging::HICON;

    // Method 1: WM_GETICON with ICON_BIG (1)
    let mut result: usize = 0;
    let sent = SendMessageTimeoutW(
        hwnd,
        WM_GETICON,
        windows::Win32::Foundation::WPARAM(1), // ICON_BIG
        windows::Win32::Foundation::LPARAM(0),
        SMTO_ABORTIFHUNG,
        100,
        Some(&mut result as *mut usize),
    );
    if sent.0 != 0 && result != 0 {
        return Some(HICON(result as *mut std::ffi::c_void));
    }

    // Method 2: WM_GETICON with ICON_SMALL (0)
    let sent = SendMessageTimeoutW(
        hwnd,
        WM_GETICON,
        windows::Win32::Foundation::WPARAM(0), // ICON_SMALL
        windows::Win32::Foundation::LPARAM(0),
        SMTO_ABORTIFHUNG,
        100,
        Some(&mut result as *mut usize),
    );
    if sent.0 != 0 && result != 0 {
        return Some(HICON(result as *mut std::ffi::c_void));
    }

    // Method 3: GetClassLongPtrW with GCLP_HICON
    let icon = GetClassLongPtrW(hwnd, GCLP_HICON);
    if icon != 0 {
        return Some(HICON(icon as *mut std::ffi::c_void));
    }

    // Method 4: GetClassLongPtrW with GCLP_HICONSM
    let icon = GetClassLongPtrW(hwnd, GCLP_HICONSM);
    if icon != 0 {
        return Some(HICON(icon as *mut std::ffi::c_void));
    }

    None
}

/// Convert HICON to RGBA pixel data
unsafe fn icon_to_rgba(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
    size: u32,
) -> Option<Vec<u8>> {
    use windows::Win32::Graphics::Gdi::HGDIOBJ;

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return None;
    }

    // Get the color bitmap if available
    let hbitmap = if !icon_info.hbmColor.is_invalid() {
        icon_info.hbmColor
    } else {
        // Clean up mask bitmap
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
        }
        return None;
    };

    // Create a compatible DC
    let hdc = CreateCompatibleDC(None);
    if hdc.is_invalid() {
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
        }
        let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
        return None;
    }

    // Select the bitmap into the DC
    let old_bitmap = SelectObject(hdc, HGDIOBJ(hbitmap.0));

    // Set up bitmap info for 32-bit RGBA
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size as i32,
            biHeight: -(size as i32), // Negative for top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default()],
    };

    // Allocate buffer for pixel data
    let mut pixels: Vec<u8> = vec![0; (size * size * 4) as usize];

    // Get the bitmap bits
    let result = GetDIBits(
        hdc,
        hbitmap,
        0,
        size,
        Some(pixels.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    // Restore old bitmap and clean up
    let _ = SelectObject(hdc, old_bitmap);
    let _ = DeleteDC(hdc);
    if !icon_info.hbmMask.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
    }
    let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));

    if result == 0 {
        return None;
    }

    // Convert BGRA to RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Swap B and R
    }

    Some(pixels)
}

/// Check if window should be skipped based on title
fn should_skip_window(title: &str) -> bool {
    const SKIP_TITLES: &[&str] = &[
        "Program Manager",
        "Windows Input Experience",
        "Microsoft Text Input Application",
        "Settings",
        "NVIDIA GeForce Overlay",
        "Default IME",
        "MSCTFIME UI",
        "DWM Notification Window",
    ];

    // Check exact match
    if SKIP_TITLES.contains(&title) {
        return true;
    }

    // Check partial matches for common system windows
    let title_lower = title.to_lowercase();
    title_lower.contains("dwm notification")
        || title_lower.contains("desktop window manager")
}

/// Check if process should be skipped based on name
fn should_skip_process(process_name: &str) -> bool {
    const SKIP_PROCESSES: &[&str] = &[
        "window-lasso",
        "WindowLasso",
        "dwm",
        "csrss",
        "conhost",
        "ApplicationFrameHost",
        "ShellExperienceHost",
        "SystemSettings",
        "SearchHost",
        "StartMenuExperienceHost",
        "TextInputHost",
        "LockApp",
    ];

    let name_lower = process_name.to_lowercase();
    SKIP_PROCESSES.iter().any(|p| name_lower == p.to_lowercase())
}

/// Check if a window rect intersects with any monitor
fn is_window_on_any_monitor(rect: &WindowRect, monitors: &[MonitorInfo]) -> bool {
    monitors
        .iter()
        .any(|monitor| rect.intersects(&monitor.bounds))
}

/// Find which monitor a window is primarily on
fn find_window_monitor(rect: &WindowRect, monitors: &[MonitorInfo]) -> Option<String> {
    let window_center = rect.center();

    // Find monitor that contains the window center
    for monitor in monitors {
        if window_center.0 >= monitor.bounds.left
            && window_center.0 < monitor.bounds.right
            && window_center.1 >= monitor.bounds.top
            && window_center.1 < monitor.bounds.bottom
        {
            return Some(monitor.name.clone());
        }
    }

    // If center isn't on any monitor, find the one with most overlap
    monitors
        .iter()
        .filter(|m| rect.intersects(&m.bounds))
        .max_by_key(|m| {
            let overlap_left = rect.left.max(m.bounds.left);
            let overlap_right = rect.right.min(m.bounds.right);
            let overlap_top = rect.top.max(m.bounds.top);
            let overlap_bottom = rect.bottom.min(m.bounds.bottom);
            (overlap_right - overlap_left) * (overlap_bottom - overlap_top)
        })
        .map(|m| m.name.clone())
}

/// Move a window to a specific monitor, scaling appropriately, maximizing, and focusing
pub fn move_window_to_monitor(hwnd: isize, monitor: &MonitorInfo) -> Result<(), String> {
    move_window_to_monitor_with_options(hwnd, monitor, None, true, true)
}

/// Move a window to a specific monitor with configurable options
/// - source_monitor: If provided, window size will be scaled proportionally
/// - maximize: If true, the window will be maximized after moving
/// - auto_focus: If true, the window will be brought to the foreground
pub fn move_window_to_monitor_with_options(
    hwnd: isize,
    target_monitor: &MonitorInfo,
    source_monitor: Option<&MonitorInfo>,
    maximize: bool,
    auto_focus: bool,
) -> Result<(), String> {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            IsZoomed, SetForegroundWindow, SetWindowPlacement,
        };

        let hwnd_handle = HWND(hwnd as *mut std::ffi::c_void);
        let was_minimized = IsIconic(hwnd_handle).as_bool();
        let was_maximized = IsZoomed(hwnd_handle).as_bool();

        // Get current window placement (works for minimized, maximized, and normal windows)
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        GetWindowPlacement(hwnd_handle, &mut placement)
            .map_err(|e| format!("Failed to get window placement: {}", e))?;

        let current_rect = placement.rcNormalPosition;
        let mut width = current_rect.right - current_rect.left;
        let mut height = current_rect.bottom - current_rect.top;

        // Scale window size based on monitor resolution if source is provided
        if let Some(src) = source_monitor {
            let src_width = src.work_area.width() as f64;
            let src_height = src.work_area.height() as f64;
            let tgt_width = target_monitor.work_area.width() as f64;
            let tgt_height = target_monitor.work_area.height() as f64;

            let scale_x = tgt_width / src_width;
            let scale_y = tgt_height / src_height;
            let scale = scale_x.min(scale_y);

            width = (width as f64 * scale) as i32;
            height = (height as f64 * scale) as i32;
        }

        // Calculate target work area dimensions
        let work_width = target_monitor.work_area.width();
        let work_height = target_monitor.work_area.height();

        // Ensure window doesn't exceed target monitor's work area
        width = width.min(work_width);
        height = height.min(work_height);

        // Calculate new position (center of monitor's work area)
        let (center_x, center_y) = target_monitor.center();
        let new_x = (center_x - width / 2).max(target_monitor.work_area.left);
        let new_y = (center_y - height / 2).max(target_monitor.work_area.top);

        // Ensure window fits within work area (right and bottom edges)
        let new_x = new_x.min(target_monitor.work_area.right - width);
        let new_y = new_y.min(target_monitor.work_area.bottom - height);

        // Update the placement's normal position
        placement.rcNormalPosition = RECT {
            left: new_x,
            top: new_y,
            right: new_x + width,
            bottom: new_y + height,
        };

        // For maximized windows: we must first restore to move them, then re-maximize
        // Windows ties the maximized state to a specific monitor, so we can't just
        // move a maximized window directly to another monitor
        if was_maximized {
            // Step 1: Restore the window first (this un-maximizes it)
            placement.showCmd = SW_RESTORE.0 as u32;
            SetWindowPlacement(hwnd_handle, &placement)
                .map_err(|e| format!("Failed to restore window: {}", e))?;

            // Step 2: Move the window to the new position on the target monitor
            SetWindowPos(
                hwnd_handle,
                Some(HWND_TOP),
                new_x,
                new_y,
                width,
                height,
                SWP_NOZORDER | SWP_SHOWWINDOW,
            )
            .map_err(|e| format!("Failed to move window: {}", e))?;

            // Step 3: Re-maximize on the new monitor if requested
            if maximize {
                let _ = ShowWindow(hwnd_handle, SW_MAXIMIZE);
            }
        } else if was_minimized {
            // For minimized windows: update the restore position and show command
            let show_cmd = if maximize { SW_MAXIMIZE } else { SW_RESTORE };
            placement.showCmd = show_cmd.0 as u32;
            SetWindowPlacement(hwnd_handle, &placement)
                .map_err(|e| format!("Failed to set window placement: {}", e))?;
        } else {
            // For normal windows: move directly, then optionally maximize
            SetWindowPos(
                hwnd_handle,
                Some(HWND_TOP),
                new_x,
                new_y,
                width,
                height,
                SWP_NOZORDER | SWP_SHOWWINDOW,
            )
            .map_err(|e| format!("Failed to move window: {}", e))?;

            if maximize {
                let _ = ShowWindow(hwnd_handle, SW_MAXIMIZE);
            }
        }

        // Bring to front if auto_focus is enabled
        if auto_focus {
            let _ = SetForegroundWindow(hwnd_handle);
        }

        Ok(())
    }
}

/// Focus this application's window (bring to foreground)
pub fn focus_self() {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetForegroundWindow, ShowWindow, SW_RESTORE,
        };
        use windows::Win32::System::Threading::GetCurrentProcessId;

        let current_pid = GetCurrentProcessId();

        // Find our window by enumerating and matching process ID
        let found_hwnd: Mutex<Option<HWND>> = Mutex::new(None);

        let _ = EnumWindows(
            Some(find_own_window_callback),
            LPARAM(&(current_pid, &found_hwnd) as *const _ as isize),
        );

        if let Some(hwnd) = found_hwnd.into_inner().unwrap_or(None) {
            // Restore if minimized
            let _ = ShowWindow(hwnd, SW_RESTORE);
            
            // Bring to foreground
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

/// Get the currently focused (foreground) window handle
pub fn get_foreground_window() -> Option<isize> {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            None
        } else {
            Some(hwnd.0 as isize)
        }
    }
}

/// Center a window on its current monitor
pub fn center_window(hwnd: isize, monitors: &[MonitorInfo]) -> Result<(), String> {
    unsafe {
        let hwnd_handle = HWND(hwnd as *mut std::ffi::c_void);
        
        // Get current window rect
        let mut rect = RECT::default();
        GetWindowRect(hwnd_handle, &mut rect)
            .map_err(|e| format!("Failed to get window rect: {}", e))?;
        
        let window_rect = WindowRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        };
        
        // Find which monitor the window is on
        let monitor = find_window_monitor(&window_rect, monitors)
            .and_then(|name| monitors.iter().find(|m| m.name == name))
            .or_else(|| monitors.iter().find(|m| m.is_primary))
            .ok_or_else(|| "No monitor found".to_string())?;
        
        let width = window_rect.width();
        let height = window_rect.height();
        
        // Calculate centered position
        let (center_x, center_y) = monitor.center();
        let new_x = center_x - width / 2;
        let new_y = center_y - height / 2;
        
        // Ensure window stays within work area
        let new_x = new_x.max(monitor.work_area.left).min(monitor.work_area.right - width);
        let new_y = new_y.max(monitor.work_area.top).min(monitor.work_area.bottom - height);
        
        SetWindowPos(
            hwnd_handle,
            Some(HWND_TOP),
            new_x,
            new_y,
            width,
            height,
            SWP_NOZORDER | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("Failed to center window: {}", e))?;
        
        Ok(())
    }
}

/// Move a window to the next monitor in the list
pub fn move_to_next_monitor(hwnd: isize, monitors: &[MonitorInfo]) -> Result<(), String> {
    if monitors.is_empty() {
        return Err("No monitors available".to_string());
    }
    
    if monitors.len() == 1 {
        return Ok(()); // Only one monitor, nothing to do
    }
    
    unsafe {
        let hwnd_handle = HWND(hwnd as *mut std::ffi::c_void);
        
        // Get current window rect
        let mut rect = RECT::default();
        GetWindowRect(hwnd_handle, &mut rect)
            .map_err(|e| format!("Failed to get window rect: {}", e))?;
        
        let window_rect = WindowRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        };
        
        // Find current monitor index
        let current_idx = find_window_monitor(&window_rect, monitors)
            .and_then(|name| monitors.iter().position(|m| m.name == name))
            .unwrap_or(0);
        
        // Get next monitor (cycle around)
        let next_idx = (current_idx + 1) % monitors.len();
        let next_monitor = &monitors[next_idx];
        
        // Move to next monitor
        move_window_to_monitor_with_options(hwnd, next_monitor, Some(&monitors[current_idx]), false, true)
    }
}

/// Callback to find our own window
unsafe extern "system" fn find_own_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let (target_pid, found_hwnd): &(u32, &Mutex<Option<HWND>>) =
        &*(lparam.0 as *const (u32, &Mutex<Option<HWND>>));

    let mut window_pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

    if window_pid == *target_pid && IsWindowVisible(hwnd).as_bool() {
        // Check if it's a main window (not a tool window)
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0) == 0 {
            if let Ok(mut guard) = found_hwnd.lock() {
                *guard = Some(hwnd);
            }
            return BOOL(0); // Stop enumeration
        }
    }

    TRUE // Continue enumeration
}
