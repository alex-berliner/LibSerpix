use cbor::Decoder;
use image::{imageops::crop_imm, ImageBuffer, Rgba};
use rustc_serialize::json::ToJson;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tokio::task;
use tokio::sync::mpsc::Sender;
use std::time::Instant;
use crate::*;

static CAPTURE_MAX_W: u32 = 900;
static CAPTURE_MAX_H: u32 = 8;

fn color_to_integer(pixel: &Rgba<u8>) -> u32 {
    let r = pixel[0] as u32;
    let g = pixel[1] as u32;
    let b = pixel[2] as u32;
    r * 256 * 256 + g * 256 + b
}

fn decode_header(header: u32) -> (u16, u8) {
    let checksum = (header & 0xFF) as u8;
    let size = ((header >> 8) & 0xFFFF) as u16;

    (size, checksum)
}

// validate one pixel
fn pixel_validate_get(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, height: u32)
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

fn dump(a: &Vec<u8>) {
    for b in a.iter() {
        print!("{:#02X} ", b);
    }
    eprintln!("\n{} bytes summed", a.len());
}

struct RxBytes {
    b: Vec<u8>,
    checksum: u8,
}

impl RxBytes {
    pub fn new(b: Vec<u8>) -> Self {
        let checksum = b.iter().fold(0, |acc, x| (acc + *x as u32)%256) as u8;
        Self { b, checksum }
    }
}

struct Frame {
    size: u16,
    checksum: u8,
    height: u32,
    // clock: u8,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Frame {
    #[allow(dead_code)]
    fn save(&self) {
        let posix_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file_name = posix_time.to_string();
        file_name.push_str(".bmp");
        self.img.save(&file_name).unwrap();
        eprintln!("Save {}", file_name);
    }

    fn pixel_validate_get(&self, x: u32) -> Result<Rgba<u8>, &'static str> {
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

    fn get_payload_pixels(&self) -> Result<Vec<Rgba<u8>>, &'static str> {
        let num_pixels = (self.size as f64/3.0).ceil() as u32;
        let pix_vec: Result<Vec<_>, _> =
            (1..=num_pixels)
            .map(|i| self.pixel_validate_get(Frame::i2p(i as u32)) )
            .collect();
        pix_vec
    }

    fn get_payload(&self) -> Result<Vec<u8>, &'static str> {
        let myvec = match self.get_payload_pixels() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("payload err {}", e);
                return Err("payload err");
            }
        };
        let mut bytevec: Vec<u8> = Vec::new();
        for p in myvec.iter() {
            bytevec.push(p[0]);
            bytevec.push(p[1]);
            bytevec.push(p[2]);
        }
        // remove padding bytes
        while bytevec.len() != self.size as usize{
            bytevec.pop();
        }
        Ok(bytevec)
    }
}

fn get_screen(hwnd: isize, w: u32, h: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let buf = win_screenshot::capture::capture_window(hwnd, win_screenshot::capture::Area::Full).unwrap();
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    crop_imm(&img, 0, 0, w, h).to_image()
}

fn frame_from_imgbuf(img: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Frame, &'static str>{
    let pixel = match pixel_validate_get(&img, 0, CAPTURE_MAX_H) {
        Ok(o) => o,
        Err(e) => {
            return Err("bad header pixel".into());
        }
    };
    let header = color_to_integer(&pixel);
    let (size, checksum_rx) = decode_header(header);
    Ok(Frame {
        size: size,
        checksum: checksum_rx,
        height: CAPTURE_MAX_H,
        // clock: clock,
        img: img
    })
}

fn cbor_parse(b: &Vec<u8>) -> Result<serde_json::Value, &'static str> {
    let mut d = Decoder::from_bytes(b.clone());
    let cbor_in = match d.items().next() {
        Some(o) => o,
        None => {
            return Err("cbor fail 1");
        }
    };
    let cbor = match cbor_in {
        Ok(o) => o,
        Err(e) => {
            eprintln!("cbor fail: {}", e);
            return Err("cbor fail 2");
        }
    };
    let c2j = cbor.to_json();
    Ok(serde_json::from_str(&c2j.to_string()).unwrap())
}

async fn screen_proc(s: ImageBuffer<Rgba<u8>, Vec<u8>>, tx: Sender<serde_json::Value>) {
    // let duration = start.elapsed();
    let frame = match frame_from_imgbuf(s) {
        Ok(v) => v,
        Err(e) => { eprintln!("frame decode error {}", e); return; },
    };

    let payload = match frame.get_payload() {
        Err(e) => { frame.save(); return; },
        Ok(v) => v,
    };

    let w = RxBytes::new(payload);
    if frame.checksum != w.checksum {
        eprintln!("checksum doesn't match rx {:#02X} calc {:#02X}",
            frame.checksum,
            w.checksum);
        if frame.checksum == w.checksum + 1 {
            eprintln!("off by 1");
            frame.save();
        }
        return;
    }

    let mut value: serde_json::Value = match cbor_parse(&w.b) {
        Ok(v) => v,
        Err(e) => {
            frame.save();
            eprintln!("{}", e);
            return;
        }
    };

    if value.is_object() {
        match value.as_object_mut() {
            None => { eprintln!("no private field, very fishy"); },
            Some(v) => {v.remove_entry("p");},
        };

        tx.send(value).await.expect("json send failed");
    }
}

pub async fn read_wow(hwnd: isize, tx: Sender<serde_json::Value>) {
    let alpha = 0.1;
    let mut avg_duration : f64 = 0.0;
    loop {
        match hwnd_exists(hwnd) {
            WindowStatus::Destroyed => break,
            WindowStatus::Minimized => {
                thread::sleep(Duration::from_millis(1));
                continue;
            },
            _ => {},
        }

        let start = Instant::now();
        let s = get_screen(hwnd, CAPTURE_MAX_W, CAPTURE_MAX_H as u32);
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            screen_proc(s, tx_clone).await;
        });
        let duration = start.elapsed().as_secs_f64();
        avg_duration = alpha * duration + (1.0 - alpha) * avg_duration;
        eprintln!("{:?}", avg_duration);
    }
}

fn find_key_start(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>, color: [u8; 3]) -> Option<(u32, u32)> {
    let (width, height) = buffer.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = buffer.get_pixel(x, y);
            if pixel[0] == color[0] && pixel[1] == color[1] && pixel[2] == color[2] {
                return Some((x, y));
            }
        }
    }

    None
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn find_key_start_test() {
        let img = image::open("assets/purple.bmp").unwrap().into_rgba8();
        assert_eq!(Some((9,38)), find_key_start(&img, [42,0,69]));
    }

    #[tokio::test]
    async fn test_profile_region_screenshot() {
        let img = image::open("assets/longstring.png").unwrap();
        let frame = match frame_from_imgbuf(img.to_rgba8()) {
            Ok(v) => v,
            Err(e) => { eprintln!("frame decode error {}", e); assert!(false); return; },
        };
        let w = RxBytes::new(frame.get_payload().unwrap());
        dump(&w.b);
        eprintln!("frame.size: {}", frame.size);
        if frame.checksum != w.checksum {
            eprintln!("checksum doesn't match rx {:#02X} calc {:#02X}",
                frame.checksum,
                w.checksum);
            dump(&w.b);
            assert!(false);
            return;
        }
        let value: serde_json::Value = match cbor_parse(&w.b) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("frame decode error {}", e);
                assert!(false);
                return;
            },
        };
    }
}

