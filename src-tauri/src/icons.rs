#[cfg(windows)]
use crate::browser::BrowserRegistration;
#[cfg(windows)]
use tauri::image::Image;

#[cfg(windows)]
const MENU_ICON_SIZE: i32 = 16;

#[cfg(windows)]
pub fn browser_icon(browser: &BrowserRegistration) -> Option<Image<'static>> {
    browser
        .icon_path
        .as_deref()
        .and_then(browser_icon_from_path)
        .or_else(|| {
            browser
                .executable_path
                .as_deref()
                .and_then(browser_icon_from_executable)
        })
        .or_else(default_browser_icon)
}

#[cfg(windows)]
fn browser_icon_from_path(path: &str) -> Option<Image<'static>> {
    Image::from_path(path).ok().map(Image::to_owned)
}

#[cfg(windows)]
pub fn browser_icon_from_executable(path: &str) -> Option<Image<'static>> {
    let hicon = load_small_icon(path)?;
    let rgba = hicon_to_rgba(hicon, MENU_ICON_SIZE, MENU_ICON_SIZE).ok()?;

    Some(Image::new_owned(
        rgba,
        MENU_ICON_SIZE as u32,
        MENU_ICON_SIZE as u32,
    ))
}

#[cfg(not(windows))]
pub fn browser_icon(_browser: &crate::browser::BrowserRegistration) -> Option<()> {
    None
}

#[cfg(not(windows))]
pub fn browser_icon_from_executable(_path: &str) -> Option<()> {
    None
}

#[cfg(windows)]
fn load_small_icon(path: &str) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
            UI::{
                Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON},
                WindowsAndMessaging::HICON,
            },
        },
    };

    let mut file_info = SHFILEINFOW::default();
    let wide_path = widestring(path);
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        )
    };

    if result == 0 || file_info.hIcon == HICON::default() {
        return None;
    }

    Some(file_info.hIcon)
}

#[cfg(windows)]
fn hicon_to_rgba(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
    width: i32,
    height: i32,
) -> windows::core::Result<Vec<u8>> {
    use windows::Win32::{
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL},
    };

    let screen_dc = unsafe { GetDC(None) };
    let memory_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    let mut bits = std::ptr::null_mut();
    let mut bitmap_info = BITMAPINFO::default();
    bitmap_info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };

    let bitmap = unsafe {
        CreateDIBSection(
            Some(screen_dc),
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            None,
            0,
        )?
    };
    let previous = unsafe { SelectObject(memory_dc, bitmap.into()) };

    unsafe {
        DrawIconEx(
            memory_dc,
            0,
            0,
            hicon,
            width,
            height,
            0,
            None,
            DI_NORMAL,
        )?;
    }

    let byte_len = (width * height * 4) as usize;
    let bgra = unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), byte_len) };
    let mut rgba = Vec::with_capacity(byte_len);
    for pixel in bgra.chunks_exact(4) {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }

    unsafe {
        SelectObject(memory_dc, previous);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory_dc);
        ReleaseDC(None, screen_dc);
        DestroyIcon(hicon).ok();
    }

    Ok(rgba)
}

#[cfg(windows)]
fn widestring(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn default_browser_icon() -> Option<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/browser-default.ico"))
        .ok()
        .map(Image::to_owned)
}
