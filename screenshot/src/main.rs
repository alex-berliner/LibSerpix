use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread;
use devtimer::run_benchmark;

use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use rustc_serialize::json::{Json, ToJson};
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

// fn pixel_validate_get(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32) -> Result<Rgba<u8>, &'static str> {
//     let pixels = (0..3)
//         .filter_map(|y| Some(img.get_pixel(x, y as u32)))
//         .collect::<Vec<_>>();

//     if all_values_equal(&pixels) {
//         Ok(*pixels[0])
//     } else {
//         Err("HEADER Not all values in the Vec are equal")
//     }
// }

fn pixel_validate_get(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, height: u8) -> Result<Rgba<u8>, &'static str> {
    let pixels = (0..height)
        .filter_map(|y| Some(img.get_pixel(x, y as u32)))
        .collect::<Vec<_>>();

    let mut counts = std::collections::HashMap::new();
    for pixel in pixels.iter() {
        *counts.entry(pixel).or_insert(0) += 1;
    }

    let mut most_common_pixel = &pixels[0];
    let mut most_common_count = 0;
    for (pixel, count) in counts.iter() {
        if count > &most_common_count {
            most_common_pixel = pixel;
            most_common_count = *count;
        }
    }

    if most_common_count >= 2 {
        Ok(*most_common_pixel.clone())
    } else {
        // self.save();
        Err("FRAME Not at least 2 pixels are the same")
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
    pub fn save(&mut self) {
        let posix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file_name = posix_time.to_string();
        file_name.push_str(".bmp");
        self.img.save(file_name).unwrap();
    }
    fn pixel_validate_get(&mut self, x: u32) -> Result<Rgba<u8>, &'static str> {
        let pixels = (0..self.height)
            .filter_map(|y| Some(self.img.get_pixel(x, y as u32)))
            .collect::<Vec<_>>();

        let mut counts = std::collections::HashMap::new();
        for pixel in pixels.iter() {
            *counts.entry(pixel).or_insert(0) += 1;
        }

        let mut most_common_pixel = &pixels[0];
        let mut most_common_count = 0;
        for (pixel, count) in counts.iter() {
            if count > &most_common_count {
                most_common_pixel = pixel;
                most_common_count = *count;
            }
        }

        if most_common_count >= 2 {
            Ok(*most_common_pixel.clone())
        } else {
            self.save();
            Err("FRAME Not at least 2 pixels are the same")
        }
    }

    fn is_data_pixel(i: u32) -> bool {
        let x = i%5;
        x == 0 || x == 3
    }

    pub fn get_all_pixels(&mut self) -> Result<Vec<Rgba<u8>>, &'static str> {
        let mut pix_vec = Vec::new();
        let mut num_pixels = (self.size as f64/3.0).ceil() as u32;
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
                    println!("{}", i);
                    return Err(e);
                }
            };
            pix_vec.push(pixel);
            if num_pixels < 1 {
                break;
            }
        }
        if num_pixels > 0 {
            println!("Expected {} pixels, got {}", (self.size as f64/3.0).ceil() as u32, (self.size as f64/3.0).ceil() as u32-num_pixels);
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

fn read_wow(tx: Sender<Json>) {
    let hwnd = find_window("World of Warcraft").unwrap();
    let mut clock_old:u32 = 9999;
    let mut total_packets = 1.0;
    let mut good_packets = 1.0;
    let pixel_height:u8 = 6;
    loop {
        let s = capture_window(hwnd, Area::Full, 400, pixel_height as i32).unwrap();
        // make dependent on pixel width somehow to avoid errors when changing size
        let pixel = match pixel_validate_get(&s, 0, pixel_height) {
            Ok(o) => o,
            Err(e) => { println!("bad header pixel"); total_packets = total_packets + 1.0; continue; }
        }; //s.get_pixel(0,0);
        let header = color_to_integer(&pixel);
        let (size, checksum_rx, clock) = decode_header(header);
        // println!("{}", size);
        let mut frame = Frame {
            size: size,
            checksum: checksum_rx,
            clock: clock,
            width: 1,
            height: pixel_height,
            img: s
        };
        if clock_old == clock.into() {
            // not necessary to warn, rust just reads really fast
            // println!("same clock clock_old {} clock {}", clock_old , clock );
            continue;
        }
        total_packets = total_packets + 1.0;
        let myvec = match frame.get_all_pixels() {
            Ok(o) =>  {/* println!("good frame"); */ o },
            Err(e) => { println!("{}", e); continue; }
        };
        let mut bytevec: Vec<u8> = Vec::new();
        for p in myvec.iter() {
            bytevec.push(p[0]);
            bytevec.push(p[1]);
            bytevec.push(p[2]);
        }
        // remove bytes padded from pixels always being 3 bytes
        while bytevec.len() > size.into() {
            bytevec.pop();
        }
        let mut checksum: u32 = 0;
        for b in bytevec.iter() {
            checksum = (checksum+*b as u32)%256;
        }
        if frame.checksum as u32 != checksum {
            println!("checksum doesn't match");
            continue;
        }
        good_packets = good_packets + 1.0;
        // println!("good packets: {}", good_packets/total_packets);
        // hex_dump(&bytevec);
        let mut d = Decoder::from_bytes(bytevec);
        let cbor = match d.items().next().unwrap() {
            Ok(o) => o,
            Err(e) => {println!("{}", e); continue;}
        };
        // println!("{}", cbor.to_json()["healing"].to_string());
        let healing = cbor.to_json()["healing"].as_u64().unwrap();
        let overhealing = cbor.to_json()["overhealing"].as_u64().unwrap();
        if healing != 0 || overhealing != 0{
            // println!("{:?}", cbor.to_json());
        }
        clock_old = clock.into();
        tx.send(cbor.to_json());
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        read_wow(tx);
    });
    for received in rx {
        println!("outside thread got: {:?}", received);
    }
}
