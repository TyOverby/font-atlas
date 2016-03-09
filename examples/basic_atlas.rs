extern crate font_atlas;
use font_atlas::*;

fn main() {
    let bytes = include_bytes!("Gudea-Regular.ttf");
    let font = load_font_from_bytes(bytes.to_vec());
    let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let atlas = font.make_atlas(alphabet.chars(), 20.0, 1, 256, 256);
    for line in atlas.lines() {
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
