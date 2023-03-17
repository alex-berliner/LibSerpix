use tokio::sync::mpsc::{channel, error};
use wow_serdes::*;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(100);
    let mut handles = vec![];
    let h = tokio::spawn(async move {
        let hwnd = win_screenshot::utils::find_window("World of Warcraft").expect("Couldn't find window");
        read_wow(hwnd, tx).await;
    });
    handles.push(h);
    let h = tokio::spawn(async move {
        let mut ctr = 0;
        loop {
            match rx.try_recv() {
                Ok(v) => {
                    let jstring = v.to_string();
                    eprintln!("payload #{}", ctr);
                    println!("{}",jstring);
                    ctr += 1;
                },
                Err(e) => {
                    match e {
                        error::TryRecvError::Disconnected => break,
                        _ => {}
                    }
                }
            }
        }
    });
    handles.push(h);
    for handle in handles {
        handle.await.expect("Thread exited");
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
