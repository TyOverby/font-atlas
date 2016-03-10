extern crate rusttype;

use std::io::Result as IoResult;
use std::path::Path;

pub mod glyph_packer;
pub mod rasterize;
pub mod cache;

pub fn load_font<P: AsRef<Path>>(path: P) -> IoResult<rasterize::Font> {
    use std::io::Read;
    use std::fs::File;
    let mut buf = vec![];
    try!(try!(File::open(path)).read_to_end(&mut buf));
    let font_collection = rusttype::FontCollection::from_bytes(buf);
    Ok(rasterize::Font::new(font_collection.into_font().unwrap()))
}

pub fn load_font_from_bytes(bytes: Vec<u8>) -> rasterize::Font {
    rasterize::Font::new(rusttype::FontCollection::from_bytes(bytes).into_font().unwrap())
}

