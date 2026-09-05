use std::env;
use std::time::Duration;

fn main() {
    let loop_mode = env::args().any(|a| a == "--loop" || a == "-l");
    if loop_mode {
        println!("Mova o mouse. Ctrl+C para sair. (poll a cada 200ms)");
        loop {
            match mouse_coords::get_position() {
                Ok(p) => println!("x={} y={}", p.x, p.y),
                Err(e) => eprintln!("erro: {e}"),
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    } else {
        match mouse_coords::get_position() {
            Ok(p) => println!("x={} y={}", p.x, p.y),
            Err(e) => {
                eprintln!("erro: {e}");
                std::process::exit(1);
            }
        }
    }
}
