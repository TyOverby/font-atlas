extern crate rusttype;
extern crate void;

use std::io::Result as IoResult;
use std::path::Path;

/// Contains methods and structures for packing characters into an atlas.
pub mod glyph_packer;
/// Contains methods and structures for rasterizing font files to bitmaps.
pub mod rasterize;
/// Contains methods and structures for caching fonts and faces.
pub mod cache;

/// An array of all of the ascii printable characters.
pub const ASCII: &'static [char] = &[
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    ':', ';', '<', '=', '>', '?', '[', ']', '\\', '|', '{', '}', '^', '~', '_', '@',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// Loads a font from a location on the file system.
pub fn load_font<P: AsRef<Path>>(path: P) -> IoResult<rasterize::Font> {
    use std::io::Read;
    use std::fs::File;
    let mut buf = vec![];
    try!(try!(File::open(path)).read_to_end(&mut buf));
    Ok(load_font_from_bytes(buf))
}

/// Loads a font from bytes in memory.
pub fn load_font_from_bytes(bytes: Vec<u8>) -> rasterize::Font {
    rasterize::Font::new(rusttype::FontCollection::from_bytes(bytes).into_font().unwrap())
}

