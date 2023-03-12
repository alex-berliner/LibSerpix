use tokio::sync::mpsc::{Sender, Receiver, channel};
use wow_serdes::*;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(100);
    let mut handles = vec![];
    let h = tokio::spawn(async move {
        read_wow(tx).await;
    });
    // println!("spawn");
    handles.push(h);
    let h = tokio::spawn(async move {
        loop {
            // println!("poll");
            match rx.recv().await {
                Some(v) =>  { println!("{:?}", v) },
                None => { println!("None") },
            }
        }
        // while .await.unwrap() { }
    });
    // println!("spawn");
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