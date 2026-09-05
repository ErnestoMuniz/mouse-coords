use crate::{Error, Point};

pub fn get_position() -> Result<Point, Error> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut pt = POINT { x: 0, y: 0 };
    // SAFETY: valid, aligned pointer; the API does not retain it.
    unsafe { GetCursorPos(&mut pt) }.map_err(|e| Error::Query(format!("GetCursorPos: {e}")))?;
    Ok(Point { x: pt.x, y: pt.y })
}
