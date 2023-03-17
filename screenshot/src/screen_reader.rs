use cbor::Decoder;
use image::{imageops::crop_imm, ImageBuffer, Rgba};
use rustc_serialize::json::ToJson;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
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

    if most_common_count >= 3 {
        Ok(*most_common_pixel)
    } else {
        Err("FRAME: Not at least 3 pixels are the same")
    }
}

struct Frame {
    size: u8,
    checksum: u8,
    height: u8,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Frame {
    #[allow(dead_code)]
    fn save(&mut self) {
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
    When instructed to draw pixels 1 space apart, WoW draws them in the sequence
    0, 3, 5, 8, 10, 13... This function models that for iteration.
    */
    fn i2p(i: u32) -> u32 {
        let r = i%2;
        let d = i/2;
        let mut v = d * 5;
        if r == 1 {
            v += 3;
        }
        v
    }

    fn get_payload_pixels(&mut self) -> Result<Vec<Rgba<u8>>, &'static str> {
        let num_pixels = (self.size as f64/3.0).ceil() as u32;
        let pix_vec: Result<Vec<_>, _> =
            (1..=num_pixels)
            .map(|i| self.pixel_validate_get(Frame::i2p(i as u32)) )
            .collect();
        pix_vec
    }
}

fn get_screen(hwnd: isize, w: u32, h: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let buf = win_screenshot::capture::capture_window(hwnd, win_screenshot::capture::Area::Full).unwrap();
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    crop_imm(&img, 0, 0, w, h).to_image()
}

pub async fn read_wow(hwnd: isize, tx: Sender<serde_json::Value>) {
    let mut clock_old:u32 = u32::MAX;
    let mut total_packets = 0;
    let mut good_packets = 0;
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
        let s = get_screen(hwnd, 400, pixel_height as u32);

        total_packets += 1;
        let pixel = match pixel_validate_get(&s, 0, pixel_height) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("bad header pixel {}", e);
                continue;
            }
        };
        let header = color_to_integer(&pixel);
        let (size, checksum_rx, clock) = decode_header(header);
        let mut frame = Frame {
            size: size,
            checksum: checksum_rx,
            height: pixel_height,
            img: s
        };
        if clock_old == clock as u32 {
            total_packets -= 1;
            continue;
        }
        let myvec = match frame.get_payload_pixels() {
            Ok(o) =>  { o },
            Err(e) => { eprintln!("payload err {}", e); continue; }
        };
        let mut bytevec: Vec<u8> = Vec::new();
        for p in myvec.iter() {
            bytevec.push(p[0]);
            bytevec.push(p[1]);
            bytevec.push(p[2]);
        }
        // remove padding bytes
        let bytevec = &bytevec[..size.into()];

        let checksum = bytevec.iter().fold(0, |acc, x| (acc + *x as u32)%256);
        if frame.checksum as u32 != checksum {
            eprintln!("checksum doesn't match");
            continue;
        }
        good_packets += 1;
        eprintln!("{} {} {}",((total_packets - good_packets) as f32) / total_packets as f32, total_packets, good_packets);
        let mut d = Decoder::from_bytes(bytevec);
        let cbor_in = match d.items().next() {
            Some(o) => o,
            None => {eprintln!("cbor fail"); frame.save(); continue;}
        };
        let cbor = match cbor_in {
            Ok(o) => o,
            Err(e) => {eprintln!("cbor fail{}", e); frame.save(); continue;}
        };
        let c2j = cbor.to_json();
        let value: serde_json::Value =
                        serde_json::from_str(&c2j.to_string()).unwrap();
        if value.is_object() {
            tx.send(value).await.expect("json send failed");
        }
        clock_old = clock.into();
        if total_packets > 10000 {
            total_packets = 0;
            good_packets = 0;
        }
    }
}
