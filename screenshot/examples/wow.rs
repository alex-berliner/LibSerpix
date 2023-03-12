use serde_json;

use tokio::sync::mpsc::{Sender, Receiver, channel};
use wow_serdes::*;
use serde_json::{Result, Value};


#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(100);
    let mut handles = vec![];
    let h = tokio::spawn(async move {
        read_wow(tx).await;
    });
    handles.push(h);
    let h = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(v) =>  {
                    // let serde_json_value = serde_json::to_value(&v).unwrap();
                    // println!("{:?}", v.to_string());
                    let value: Value = serde_json::from_str(&v.to_string()).unwrap();
                    println!("{}", value.to_string());
                },
                None => { println!("None") },
            }
        }
    });
    handles.push(h);
    for handle in handles {
        handle.await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    async fn profile_stock_screenshot() {
        let hwnd = find_window("World of Warcraft").expect("Couldn't find window");
        let s = win_screenshot::capture::capture_window(hwnd, win_screenshot::capture::Area::Full).expect("Couldn't capture window");
    }

    async fn profile_region_screenshot() {
        let hwnd = find_window("World of Warcraft").expect("Couldn't find window");
        let s = wow_serdes::capture_window(hwnd, wow_serdes::Area::Full, 400, 6).expect("Couldn't capture window");
    }

    #[tokio::test]
    async fn test_profile_region_screenshot() {
        let start_time = Instant::now();
        for i in 0..100 {
            profile_region_screenshot().await;
        }
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("test_profile_region_screenshot {:?}", duration);
    }
    #[tokio::test]
    async fn test_profile_stock_screenshot() {
        let start_time = Instant::now();
        for i in 0..100 {
            profile_stock_screenshot().await;
        }
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("test_profile_stock_screenshot {:?}", duration);
    }
}
