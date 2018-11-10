extern crate winapi;

use std::env;

use std::fmt;
use std::i64;
use std::io::Error;
use std::mem;
use std::process;

use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::shared::windef::{HWND, LPRECT, RECT};
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use winapi::um::winnt::{LPWSTR, WCHAR};
use winapi::um::winuser::{GetWindowRect, GetWindowTextLengthW, GetWindowTextW};

#[derive(Debug)]
struct PrettyPrintRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl fmt::Display for PrettyPrintRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}), ({}, {}) - {}x{}",
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.right - self.left,
            self.bottom - self.top
        )
    }
}

impl From<RECT> for PrettyPrintRect {
    fn from(rect: RECT) -> Self {
        PrettyPrintRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

fn dwm_get_window_rect(hwnd: i32) -> Result<PrettyPrintRect, Error> {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ret = unsafe {
        DwmGetWindowAttribute(
            hwnd as HWND,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut RECT as LPVOID,
            mem::size_of::<RECT>() as DWORD,
        )
    };
    if SUCCEEDED(ret) {
        Ok(PrettyPrintRect::from(rect))
    } else {
        Err(Error::from_raw_os_error(ret))
    }
}

fn get_window_rect(hwnd: i32) -> Result<PrettyPrintRect, Error> {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ret = unsafe { GetWindowRect(hwnd as HWND, &mut rect as LPRECT) };
    if ret != 0 {
        Ok(PrettyPrintRect::from(rect))
    } else {
        Err(Error::from_raw_os_error(ret))
    }
}

fn get_window_title_text(hwnd: i32) -> Result<String, Error> {
    unsafe {
        let text_len = GetWindowTextLengthW(hwnd as HWND);
        if text_len > 0 {
            let max_size: usize = text_len as usize + 1;
            // https://www.reddit.com/r/rust/comments/6wra5d/need_help_with_kernel32readprocessmemory/
            let mut wide: Vec<WCHAR> = Vec::with_capacity(max_size);
            wide.set_len(max_size);
            let ret = GetWindowTextW(hwnd as HWND, wide.as_ptr() as LPWSTR, max_size as i32);
            if ret == 0 {
                return Err(Error::last_os_error());
            } else {
                return Ok(String::from_utf16_lossy(wide.as_ref()));
            }
        }
    }
    Ok(String::from(""))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing argument: HWND value");
        process::exit(1);
    }
    let hwnd: i32 =
        i64::from_str_radix(args[1].trim(), 16).expect("Please type a valid number!") as i32;
    //
    let title_text = get_window_title_text(hwnd).unwrap();
    println!("title: [{}]", title_text);
    //
    let rect = dwm_get_window_rect(hwnd).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });
    println!("rect by DwmGetWindowAttribute: {}", rect);

    let rect = get_window_rect(hwnd).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });
    println!("rect by GetWindowRect: {}", rect);
}
