//! プラットフォーム固有のウィンドウ処理（アニメーション無効化など）

#[cfg(windows)]
mod windows_impl {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::sync::atomic::{AtomicPtr, Ordering};
    use windows_sys::core::BOOL;
    use windows_sys::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, TRUE, WPARAM};
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, EnumThreadWindows, GetWindowTextW, SetWindowsHookExW, CBT_CREATEWNDW,
        HHOOK, HCBT_CREATEWND, WH_CBT,
    };

    static HOOK_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// ウィンドウのDWMトランジション（表示・非表示・拡大縮小等のアニメーション）を強制的に無効化する
    pub fn disable_window_transitions(hwnd: HWND) {
        unsafe {
            let disable: BOOL = TRUE;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_TRANSITIONS_FORCEDISABLED as u32,
                &disable as *const _ as *const c_void,
                size_of::<BOOL>() as u32,
            );
        }
    }

    /// CBTフックプロシージャ: ウィンドウ生成時（表示前）にアニメーションを無効化
    unsafe extern "system" fn cbt_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code == HCBT_CREATEWND as i32 {
            let hwnd = w_param as HWND;
            let create_struct = l_param as *const CBT_CREATEWNDW;
            if !create_struct.is_null() && !(*create_struct).lpcs.is_null() {
                let cs = &*(*create_struct).lpcs;
                if !cs.lpszName.is_null() && (cs.lpszName as usize) > 0xFFFF {
                    let mut len = 0;
                    while *cs.lpszName.add(len) != 0 {
                        len += 1;
                    }
                    let slice = std::slice::from_raw_parts(cs.lpszName, len);
                    let title = String::from_utf16_lossy(slice);
                    // オーバーレイウィンドウを検出してアニメーションを無効化
                    if title.contains("Overlay") {
                        disable_window_transitions(hwnd);
                    }
                }
            }
        }

        let hook = HOOK_HANDLE.load(Ordering::SeqCst) as HHOOK;
        CallNextHookEx(hook, n_code, w_param, l_param)
    }

    /// スレッドフックを設定し、作成されるオーバーレイウィンドウのアニメーションを無効化
    pub fn init_window_animation_hook() {
        unsafe {
            let thread_id = GetCurrentThreadId();
            let hook = SetWindowsHookExW(
                WH_CBT,
                Some(cbt_hook_proc),
                0 as HMODULE,
                thread_id,
            );
            if !hook.is_null() {
                HOOK_HANDLE.store(hook as *mut c_void, Ordering::SeqCst);
            }
        }
    }

    /// スレッドフックの解除
    pub fn cleanup_window_animation_hook() {
        let hook = HOOK_HANDLE.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !hook.is_null() {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(hook as HHOOK);
            }
        }
    }

    /// タイトルに"Overlay"を含む自スレッドのウィンドウのアニメーションを無効化（バックアップ処理）
    pub fn disable_overlay_window_animations() {
        unsafe {
            unsafe extern "system" fn enum_proc(hwnd: HWND, _l_param: LPARAM) -> BOOL {
                let mut buf = [0u16; 256];
                let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
                if len > 0 {
                    let title = String::from_utf16_lossy(&buf[..len as usize]);
                    if title.contains("Overlay") {
                        disable_window_transitions(hwnd);
                    }
                }
                TRUE
            }

            EnumThreadWindows(GetCurrentThreadId(), Some(enum_proc), 0);
        }
    }
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn init_window_animation_hook() {}

#[cfg(not(windows))]
pub fn cleanup_window_animation_hook() {}

#[cfg(not(windows))]
pub fn disable_overlay_window_animations() {}
