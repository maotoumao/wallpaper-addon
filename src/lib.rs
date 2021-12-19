#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use std::convert::TryInto;

use napi::{CallContext, JsNumber, JsObject, JsString, Result};

use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("setBackgroundWindow", set_background_window)?;

    exports.create_named_method("resumeBackgroundWindow", resume_background_window)?;

    Ok(())
}

/**
 * 隐藏桌面的回调
 */
extern "system" fn lpenumfunc(hwnd: HWND, _: LPARAM) -> BOOL {
    unsafe {
        let hDefView = FindWindowExW(hwnd, HWND::default(), "SHELLDLL_DefView", PWSTR::default());
        if hDefView != HWND(0) {
            let hworkerw = FindWindowExW(HWND::default(), hwnd, "WorkerW", PWSTR::default());
            ShowWindow(hworkerw, SW_HIDE);
            return BOOL(0);
        }
    }
    return BOOL(1);
}

/**
 * 设置背景
 */
#[js_function(1)]
fn set_background_window(ctx: CallContext) -> Result<JsObject> {
    let window_name = ctx.get::<JsString>(0)?.into_utf8()?;
    unsafe {
        let scr_width = GetSystemMetrics(SYSTEM_METRICS_INDEX(0));
        let scr_height = GetSystemMetrics(SYSTEM_METRICS_INDEX(1));
        let target_hwnd = FindWindowW(PWSTR::default(), window_name.as_str()?);
        let mut lprect = RECT::default();
        GetWindowRect(target_hwnd, (&mut lprect) as *mut RECT);
        let original_style = GetWindowLongPtrW(target_hwnd, GWL_STYLE);
        SetWindowLongPtrW(target_hwnd, GWL_STYLE, 0);
        ShowWindow(target_hwnd, SW_SHOW);
        MoveWindow(target_hwnd, 0, 0, scr_width, scr_height, false);
        let hwnd = FindWindowA("Progman", PSTR::default());
        SendMessageTimeoutA(
            hwnd,
            0x52C,
            WPARAM(0),
            LPARAM(0),
            SEND_MESSAGE_TIMEOUT_FLAGS(0),
            1000,
            &mut 0,
        );
        SetParent(target_hwnd, hwnd);
        EnumWindows(Some(lpenumfunc), LPARAM::default());
        let mut result = ctx.env.create_object()?;
        if target_hwnd == HWND::default() {
            result.set_named_property("success", ctx.env.get_boolean(false)?)?;
        } else {
            result.set_named_property("success", ctx.env.get_boolean(true)?)?;
            result.set_named_property("windowX", ctx.env.create_int32(lprect.left)?)?;
            result.set_named_property("windowY", ctx.env.create_int32(lprect.top)?)?;
            result.set_named_property(
                "windowWidth",
                ctx.env.create_int32(lprect.right - lprect.left)?,
            )?;
            result.set_named_property(
                "windowHeight",
                ctx.env.create_int32(lprect.bottom - lprect.top)?,
            )?;
            if isize::BITS == 64 {
                result.set_named_property(
                    "windowStyle",
                    ctx.env.create_int64(original_style as i64)?,
                )?;
            } else {
                result.set_named_property(
                    "windowStyle",
                    ctx.env.create_int32(original_style as i32)?,
                )?;
            }
        }
        return Ok(result);
    };
}

/**
 * 恢复背景
 */
#[js_function(2)]
fn resume_background_window(ctx: CallContext) -> Result<JsObject> {
    let window_name = ctx.get::<JsString>(0)?.into_utf8()?;
    let window_options = ctx.get::<JsObject>(1)?;
    unsafe {
        let hwnd = FindWindowA("Progman", PSTR::default());
        let target_hwnd = FindWindowExW(
            hwnd,
            HWND::default(),
            PWSTR::default(),
            window_name.as_str()?,
        );
        SetParent(target_hwnd, HWND::default());
        let mut window_style: isize = isize::default();
        // 恢复窗口大小
        if isize::BITS == 64 {
            let style64: i64 = window_options
                .get_named_property::<JsNumber>("windowStyle")?
                .try_into()?;
            window_style = style64 as isize;
        } else {
            let style32: i32 = window_options
                .get_named_property::<JsNumber>("windowStyle")?
                .try_into()?;
            window_style = style32 as isize;
        }
        SetWindowLongPtrW(target_hwnd, GWL_STYLE, window_style);
        ShowWindow(target_hwnd, SW_SHOW);
        MoveWindow(
            target_hwnd,
            window_options
                .get_named_property::<JsNumber>("windowX")?
                .try_into()?,
            window_options
                .get_named_property::<JsNumber>("windowY")?
                .try_into()?,
            window_options
                .get_named_property::<JsNumber>("windowWidth")?
                .try_into()?,
            window_options
                .get_named_property::<JsNumber>("windowHeight")?
                .try_into()?,
            false,
        );

        let mut result = ctx.env.create_object()?;
        if target_hwnd == HWND::default() {
            result.set_named_property("success", ctx.env.get_boolean(false)?)?;
        } else {
            result.set_named_property("success", ctx.env.get_boolean(true)?)?;
        }
        return Ok(result);
    };
}
