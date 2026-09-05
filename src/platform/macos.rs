use crate::{Error, Point};

pub fn get_position() -> Result<Point, Error> {
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| Error::Query("CGEventSource::new falhou".into()))?;
    let event = CGEvent::new(source).map_err(|_| Error::Query("CGEvent::new falhou".into()))?;
    let loc = event.location();
    Ok(Point {
        x: loc.x as i32,
        y: loc.y as i32,
    })
}
