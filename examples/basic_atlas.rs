extern crate font_atlas;
use font_atlas::*;

fn main() {
    let bytes = include_bytes!("Gudea-Regular.ttf");
    let font = load_font_from_bytes(bytes.to_vec());
    let chars = font_atlas::ASCII.iter().cloned().chain(font_atlas::ASCII.iter().cloned());
    let (_, bitmap, _) = font.make_atlas(chars, 20.0, 1, 128, 128);
    for line in bitmap.lines() {
        print!("{:03} ", line.len());
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
