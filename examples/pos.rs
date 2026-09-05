use std::env;
use std::time::Duration;

fn main() {
    let loop_mode = env::args().any(|a| a == "--loop" || a == "-l");
    if loop_mode {
        println!("Move the mouse. Ctrl+C to exit. (poll every 200ms)");
        loop {
            match mouse_coords::get_position() {
                Ok(p) => println!("x={} y={}", p.x, p.y),
                Err(e) => eprintln!("error: {e}"),
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    } else {
        match mouse_coords::get_position() {
            Ok(p) => println!("x={} y={}", p.x, p.y),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
