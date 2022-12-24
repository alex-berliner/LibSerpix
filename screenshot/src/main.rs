use std::io::{BufRead, BufReader};

use cbor::{Decoder, Encoder};

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
fn decode_header(header: u32) -> (u8, u8, u8) {
    let size = (header >> 16) as u8;
    let checksum = ((header >> 8) & 0xff) as u8;
    let clock = (header & 0xff) as u8;

    (size, checksum, clock)
}

fn all_values_equal<T: PartialEq>(vec: &Vec<T>) -> bool {
    vec.iter().all(|x| x.eq(&vec[0]))
}

fn pixel_validate_get(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32) -> Result<Rgba<u8>, &'static str> {
    let pixels = (0..3)
        .filter_map(|y| Some(img.get_pixel(x, y as u32)))
        .collect::<Vec<_>>();

    if all_values_equal(&pixels) {
        Ok(*pixels[0])
    } else {
        Err("Not all values in the Vec are equal")
    }
}

struct Frame {
    size: u8,
    checksum: u8,
    clock: u8,
    width: u8,
    height: u8,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
}
impl Frame {
    pub fn pixel_validate_get(&mut self, x: u32) -> Result<Rgba<u8>, &'static str> {
        let pixels = (0..self.height)
            .filter_map(|y| Some(self.img.get_pixel(x, y as u32)))
            .collect::<Vec<_>>();

        if all_values_equal(&pixels) {
            Ok(*pixels[0])
        } else {
            Err("Not all values in the Vec are equal")
        }
    }

    fn is_data_pixel(i: u32) -> bool {
        let x = i%5;
        x == 0 || x == 3
    }

    pub fn get_all_pixels(&mut self) -> Result<Vec<Rgba<u8>>, &'static str> {
        let mut pix_vec = Vec::new();
        let mut num_pixels = self.size/3;
        for i in 2..400 {
            if !Frame::is_data_pixel(i) {
                continue;
            }
            let pixel = match self.pixel_validate_get(i) {
                Ok(p) => {
                    num_pixels -= 1;
                    p
                },
                Err(e) => {
                    return Err(e);
                }
            };
            pix_vec.push(pixel);
            if num_pixels < 1 {
                break;
            }
        }
        if num_pixels > 0 {
            println!("Expected {} pixels, got {}", self.size/3, self.size/3-num_pixels);
            return Err("Pixels missing from image");
        }

        Ok(pix_vec)
    }
}
fn hex_dump(data: &[u8]) {
    println!("{}", data.len());
    for chunk in data.chunks(16) {
        print!("{:08x}  ", data.as_ptr() as usize);
        for &byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }
}

fn main() {
    let hwnd = find_window("World of Warcraft").unwrap();
    let mut clock_old:u32 = 9999;
    let mut total_packets = 1.0;
    let mut good_packets = 1.0;
    loop {
        let mut checksum_cx = 0;
        let mut s = capture_window(hwnd, Area::Full, 400, 3).unwrap();
        // make dependent on pixel width somehow to avoid errors when changing size
        let pixel = match pixel_validate_get(&s, 0) {
            Ok(o) => o,
            Err(e) => { println!("bad header pixel"); continue; }
        }; //s.get_pixel(0,0);
        let header = color_to_integer(&pixel);
        let (size, checksum_rx, clock) = decode_header(header);
        // println!("{}", size);
        let mut frame = Frame {
            size: size,
            checksum: checksum_rx,
            clock: clock,
            width: 1,
            height: 3,
            img: s
        };
        if clock_old == clock.into() {
            // not necessary to warn, rust just reads really fast
            // println!("same clock clock_old {} clock {}", clock_old , clock );
            continue;
        }
        total_packets = total_packets + 1.0;
        let mut myvec = match frame.get_all_pixels() {
            Ok(o) =>  {/* println!("good frame"); */ o },
            Err(e) => { /* println!("{}", e); */ continue; }
        };
        let mut u8vec: Vec<u8> = Vec::new();
        for p in myvec.iter() {
            u8vec.push(p[0]);
            u8vec.push(p[1]);
            u8vec.push(p[2]);
        }
        // hex_dump(&u8vec);
        let mut checksum: u32 = 0;
        for b in u8vec.iter() {
            checksum = (checksum+*b as u32)%256;
            // println!("{}", b);
        }
        // println!("checksum matches: {}", );
        if frame.checksum as u32 != checksum {
            // println!("checksum doesn't match");
            continue;
        }
        good_packets = good_packets + 1.0;
        println!("good packets: {}", good_packets/total_packets);
        clock_old = clock.into();
    }
}
