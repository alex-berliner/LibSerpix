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
    use super::*;

    #[tokio::test]
    async fn test_profile_region_screenshot() {
    }
}
