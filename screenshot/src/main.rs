use devtimer::run_benchmark;

use image::imageops::flip_vertical;
use image::{ImageBuffer, Rgba};
use std::mem::size_of;
use windows::Win32::Foundation::{ERROR_INVALID_PARAMETER, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    SRCCOPY,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS, PW_CLIENTONLY};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PW_RENDERFULLCONTENT, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};
use win_screenshot::addon::*;

#[derive(Debug)]
pub enum WSError {
    GetDCIsNull,
    GetClientRectIsZero,
    CreateCompatibleDCIsNull,
    CreateCompatibleBitmapIsNull,
    SelectObjectError,
    PrintWindowIsZero,
    GetDIBitsError,
    GetSystemMetricsIsZero,
    StretchBltIsZero,
}

pub enum Area {
    Full,
    ClientOnly,
}

pub type Image = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn capture_window(hwnd: isize, area: Area, width: i32, height: i32) -> Result<Image, WSError> {
    let hwnd = HWND(hwnd);

    unsafe {
        let mut rect = RECT::default();

        let hdc_screen = GetDC(hwnd);
        if hdc_screen.is_invalid() {
            return Err(WSError::GetDCIsNull);
        }

        let get_cr = match area {
            Area::Full => GetWindowRect(hwnd, &mut rect),
            Area::ClientOnly => GetClientRect(hwnd, &mut rect),
        };
        if get_cr == false {
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::GetClientRectIsZero);
        }

        let hdc = CreateCompatibleDC(hdc_screen);
        if hdc.is_invalid() {
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::CreateCompatibleDCIsNull);
        }

        let hbmp = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbmp.is_invalid() {
            DeleteDC(hdc);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::CreateCompatibleBitmapIsNull);
        }

        let so = SelectObject(hdc, hbmp);
        if so.is_invalid() {
            DeleteDC(hdc);
            DeleteObject(hbmp);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::SelectObjectError);
        }

        let bmih = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biPlanes: 1,
            biBitCount: 32,
            biWidth: width,
            biHeight: height,
            biCompression: BI_RGB as u32,
            ..Default::default()
        };

        let mut bmi = BITMAPINFO {
            bmiHeader: bmih,
            ..Default::default()
        };

        let mut buf: Vec<u8> = vec![0; (4 * width * height) as usize];

        let flags = match area {
            Area::Full => PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT),
            Area::ClientOnly => PRINT_WINDOW_FLAGS(PW_CLIENTONLY.0 | PW_RENDERFULLCONTENT),
        };
        let pw = PrintWindow(hwnd, hdc, flags);
        if pw == false {
            DeleteDC(hdc);
            DeleteObject(hbmp);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::PrintWindowIsZero);
        }

        let gdb = GetDIBits(
            hdc,
            hbmp,
            0,
            height as u32,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        if gdb == 0 || gdb == ERROR_INVALID_PARAMETER.0 as i32 {
            DeleteDC(hdc);
            DeleteObject(hbmp);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(WSError::GetDIBitsError);
        }

        buf.chunks_exact_mut(4).for_each(|c| c.swap(0, 2));

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width as u32, height as u32, buf).unwrap();

        DeleteDC(hdc);
        DeleteObject(hbmp);
        ReleaseDC(HWND::default(), hdc_screen);

        Ok(flip_vertical(&img))
    }
}

fn color_to_integer(pixel: &Rgba<u8>) -> u32 {
    let r = pixel[0] as u32;
    let g = pixel[1] as u32;
    let b = pixel[2] as u32;
    r * 256 * 256 + g * 256 + b
}

fn main() {
    let hwnd = find_window("World of Warcraft").unwrap();
    let mut old = 999999;
    loop {
        let mut s = capture_window(hwnd, Area::Full, 200, 200).unwrap();
        let pixel = s.get_pixel(5,0);
        let c2i1 = color_to_integer(pixel);
        let pixel = s.get_pixel(15,0);
        let c2i2 = color_to_integer(pixel);
        let pixel = s.get_pixel(25,0);
        let c2i3 = color_to_integer(pixel);
        if old == 999999 {
            old = c2i1;
            continue;
        }
        if old == c2i1 {
            continue;
        }
        if c2i1 != old + 1 && c2i1 != 0 {
            println!("BIG PROBLEM: {} != {}", c2i1, old+1);
        }
        if c2i1 == 0 {
            println!("tick");
        }
        old = c2i1;
        // if old != c2i {
        // println!("{} {} {} {} -> {}", pixel[0], pixel[1], pixel[2], pixel[3], c2i);
        // println!("{} {} {}", c2i1, c2i2, c2i3);
        //     old = c2i;
        // }
    }
}


// // extern crate image;
// extern crate winapi;

// use win_screenshot::addon::*;
// use win_screenshot::capture::*;

// // use image::{RgbImage, Rgb, Pixel};
// // use screenshots::Screen;

// use std::{mem, thread, time};
// use winapi::um::winuser::{GetCursorPos, GetDC};
// use winapi::um::wingdi::{GetPixel};
// use winapi::um::winuser::{GetForegroundWindow};
// // use image::{RgbImage, Rgb, Pixel};

// fn main() {
//     // Wait for 3 seconds
//     thread::sleep(time::Duration::from_secs(1));
//     let hwnd = find_window("World of Warcraft").unwrap();
//     let s = capture_window(hwnd, Area::Full).unwrap();

//     // // Get the handle to the window device context
//     // let hwnd = unsafe { GetForegroundWindow() };
//     // let hdc = unsafe { GetDC(hwnd) };

//     // let width = 100; //(window_rect.right - window_rect.left) as usize;
//     // let height = 100; //(window_rect.bottom - window_rect.top) as usize;

//     // // Retrieve the pixel colors for each pixel in the window
//     // let mut img = RgbImage::new(1920, 1080);
//     // for y in 0..height {
//     //     for x in 0..width {
//     //         let pixel_color = unsafe { GetPixel(hdc, x as i32, y as i32) };
//     //         let r = ((pixel_color & 0x00FF0000) >> 16) as u8;
//     //         let g = ((pixel_color & 0x0000FF00) >> 8) as u8;
//     //         let b = (pixel_color & 0x000000FF) as u8;
//     //         // let p = Rgb([r,g,b]);
//     //         // let pixel: Pixel = p.into();
//     //         img.put_pixel(x as u32, y as u32, Rgb([r,g,b]));
//     //     }
//     // }
//     s.save("screenshot.png").unwrap();
// }
