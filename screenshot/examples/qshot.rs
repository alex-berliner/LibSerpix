use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_handle = unsafe {
        FindWindowW(None, &windows::core::HSTRING::from("example window name"))
    };

    let manager = qshot::CaptureManager::new(
        window_handle.0,
        (0, 0),
        (500, 500)
    )?;

    Ok(())
}
