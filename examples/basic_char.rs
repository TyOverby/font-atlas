extern crate font_atlas;
use font_atlas::*;

fn main() {
    let bytes = include_bytes!("Gudea-Regular.ttf");
    let font = load_font_from_bytes(bytes.to_vec());
    let bitmap = font.render_char('A', 40.0);
    for line in bitmap.unwrap().lines() {
        for &pixel in line {
            if pixel == 0 {
                print!(" ");
            } else {
                print!("#");
            }
        }
        println!("");
    }
}
