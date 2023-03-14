use cbor::{Decoder, Encoder};
use image::{ImageBuffer, Rgba};
use rustc_serialize::json::{Json, ToJson};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use crate::*;

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

fn pixel_validate_get(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, height: u8)
                                            -> Result<Rgba<u8>, &'static str> {
    let pixels = (0..height)
        .filter_map(|y| Some(img.get_pixel(x, y as u32)))
        .collect::<Vec<_>>();

    let pixel_counts =
        pixels.iter().fold(std::collections::HashMap::new(), |mut acc, &x| {
        *acc.entry(x).or_insert(0) += 1;
        acc
    });

    let (most_common_pixel, most_common_count) = pixel_counts.iter()
        .max_by_key(|&(_, count)| count)
        .map(|(pixel, count)| (*pixel, *count))
        .unwrap();

    if most_common_count >= 2 {
        Ok(*most_common_pixel)
    } else {
        Err("FRAME Not at least 2 pixels are the same")
    }
}

struct Frame {
    size: u8,
    checksum: u8,
    height: u8,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Frame {
    pub fn save(&mut self) {
        let posix_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file_name = posix_time.to_string();
        file_name.push_str(".bmp");
        self.img.save(file_name).unwrap();
    }

    fn pixel_validate_get(&mut self, x: u32) -> Result<Rgba<u8>, &'static str> {
        pixel_validate_get(&self.img, x, self.height)
    }

    /*
    This models the way WoW draws lines of pixels do the screen when instructed
    to draw them 1 pixel apart
    */
    fn is_data_pixel(i: u32) -> bool {
        let x = i%5;
        x == 0 || x == 3
    }

    pub fn get_all_pixels(&mut self) -> Result<Vec<Rgba<u8>>, &'static str> {
        let mut pix_vec = Vec::new();
        let mut num_pixels = (self.size as f64/3.0).ceil() as u32;
        if num_pixels == 0 {
            return Err("0 pixels");
        }
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
                    // println!("{}", i);
                    return Err(e);
                }
            };
            pix_vec.push(pixel);
            if num_pixels < 1 {
                break;
            }
        }
        if num_pixels > 0 {
            // println!("Expected {} pixels, got {}",
                // (self.size as f64/3.0).ceil() as u32,
                // (self.size as f64/3.0).ceil() as u32-num_pixels);
            return Err("Pixels missing from image");
        }

        Ok(pix_vec)
    }
}

pub async fn read_wow(hwnd: isize, tx: Sender<serde_json::Value>) {
    let mut clock_old:u32 = 9999;
    let mut total_packets = 1;
    let mut good_packets = 1;
    let pixel_height: u8 = 6;
    loop {
        match hwnd_exists(hwnd) {
            WindowStatus::Destroyed => break,
            WindowStatus::Minimized => {
                thread::sleep(Duration::from_millis(1));
                continue;
            },
            _ => {},
        }

        let s =  capture_window(hwnd,
                                local_capture::Area::Full,
                                400,
                                pixel_height as i32).unwrap();
        let pixel = match pixel_validate_get(&s, 0, pixel_height) {
            Ok(o) => o,
            Err(e) => {
                // println!("bad header pixel");
                total_packets = total_packets + 1;
                continue;
            }
        };
        let header = color_to_integer(&pixel);
        let (size, checksum_rx, clock) = decode_header(header);
        // println!("{}", size);
        let mut frame = Frame {
            size: size,
            checksum: checksum_rx,
            height: pixel_height,
            img: s
        };
        if clock_old == clock as u32 {
            continue;
        }
        total_packets = total_packets + 1;
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
        // remove padding bytes
        let bytevec = &bytevec[..size.into()];
        let mut checksum: u32 = 0;
        for b in bytevec.iter() {
            checksum = (checksum+*b as u32)%256;
        }
        if frame.checksum as u32 != checksum {
            // println!("checksum doesn't match");
            continue;
        }
        good_packets = good_packets + 1;

        // println!("{}",(total_packets as f32 - good_packets as f32) / total_packets as f32);
        let mut d = Decoder::from_bytes(bytevec);
        let cbor = match d.items().next().unwrap() {
            Ok(o) => o,
            Err(e) => {println!("{}", e); continue;}
        };
        let c2j = cbor.to_json();
        let value: serde_json::Value =
                        serde_json::from_str(&c2j.to_string()).unwrap();
        if value.is_object() {
            tx.send(value).await.expect("json send failed");
        }
        clock_old = clock.into();
    }
}
