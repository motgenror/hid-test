use std::ffi::c_void;
use std::mem::size_of;

use windows::core::w;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::UI::Input::{GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTDEVICE_FLAGS, RAWINPUTHEADER, RAWKEYBOARD, RAWMOUSE, RIDEV_NOLEGACY, RID_INPUT, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE, RIDEV_INPUTSINK};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage, HMENU, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_INPUT, WNDCLASSW};

unsafe extern "system" fn window_proc(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if umsg == WM_INPUT {
        let mut size: u32 = 0;
        unsafe { GetRawInputData(HRAWINPUT(lparam.0 as *mut c_void), RID_INPUT, None, &mut size, size_of::<RAWINPUTHEADER>() as u32); }
        let mut buffer = vec![0u8; size as usize];
        unsafe {
            if GetRawInputData(HRAWINPUT(lparam.0 as *mut c_void), RID_INPUT, Some(buffer.as_mut_ptr() as _), &mut size, size_of::<RAWINPUTHEADER>() as u32) == size {
                let raw = &*(buffer.as_ptr() as *const RAWINPUT);
                let mut counter: i64 = 0;
                QueryPerformanceCounter(&mut counter);
                let mut freq: i64 = 0;
                QueryPerformanceFrequency(&mut freq);
                let timestamp = counter as f64 / freq as f64;
                if raw.header.dwType == RIM_TYPEKEYBOARD.0 {
                    let kb: RAWKEYBOARD = raw.data.keyboard;
                    println!(
                        "Keyboard: VKey={:04x}, Message={:04x}, MakeCode={:04x}, Flags={:04x}, Timestamp={:.6}",
                        kb.VKey, kb.Message, kb.MakeCode, kb.Flags, timestamp
                    );
                } else if raw.header.dwType == RIM_TYPEMOUSE.0 {
                    let mouse: RAWMOUSE = raw.data.mouse;
                    println!(
                        "Mouse: LastX={}, LastY={}, ButtonFlags={:04x}, Timestamp={:.6}",
                        mouse.lLastX, mouse.lLastY, mouse.Anonymous.Anonymous.usButtonFlags, timestamp
                    );
                }
            }
        }
    }
    unsafe { DefWindowProcW(hwnd, umsg, wparam, lparam) }
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("RawInputDemo");
        let mut wc = WNDCLASSW::default();
        wc.lpfnWndProc = Some(window_proc);
        wc.hInstance = instance.into();
        wc.lpszClassName = class_name;
        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_win32());
        }
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!(""),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU::default(),
            HMODULE(instance.0),
            None,
        )?;
        let rid = [
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06,
                dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_NOLEGACY.0 | RIDEV_INPUTSINK.0),
                hwndTarget: hwnd,
            }, // keyboard
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_NOLEGACY.0 | RIDEV_INPUTSINK.0),
                hwndTarget: hwnd,
            }, // mouse
        ];
        RegisterRawInputDevices(&rid, size_of::<RAWINPUTDEVICE>() as u32)?;
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}