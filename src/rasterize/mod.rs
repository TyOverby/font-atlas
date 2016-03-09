use std::io::Result as IoResult;
use std::path::Path;
use std::slice::Chunks;

use super::glyph_packer;
use super::rusttype;

pub struct Font {
    font: rusttype::Font<'static>
}

pub struct Bitmap {
    bytes: Vec<u8>,
    width: usize
}

pub fn load_font<P: AsRef<Path>>(path: P) -> IoResult<Font> {
    use std::io::Read;
    use std::fs::File;
    let mut buf = vec![];
    try!(try!(File::open(path)).read_to_end(&mut buf));
    let font_collection = rusttype::FontCollection::from_bytes(buf);
    Ok(Font {
        font: font_collection.into_font().unwrap()
    })
}

pub fn load_font_from_bytes(bytes: Vec<u8>) -> Font {
    Font {
        font: rusttype::FontCollection::from_bytes(bytes).into_font().unwrap()
    }
}

impl Bitmap {
    fn new(w: usize, h: usize) -> Bitmap {
        Bitmap {
            bytes: vec![0; w * h],
            width: w,
        }
    }

    pub fn lines(&self) -> Chunks<u8> {
        self.bytes.chunks(self.width)
    }
}

impl glyph_packer::Buffer2d for Bitmap {
    type Pixel = u8;

    fn width(&self) -> u32 {
        self.width as u32
    }

    fn height(&self) -> u32 {
        (self.bytes.len() / self.width) as u32
    }

    fn get(&self, x: u32, y: u32) -> Option<Self::Pixel> {
        self.bytes.get((x + self.width() * y) as usize).cloned()
    }

    fn set(&mut self, x: u32, y: u32, val: Self::Pixel) {
        let w = self.width();
        if let Some(p) = self.bytes.get_mut((x + w * y) as usize) {
            *p = val;
        }
    }
}

impl glyph_packer::ResizeBuffer for Bitmap {
    fn resize(&mut self, width: u32, height: u32) {
        use glyph_packer::Buffer2d;
        assert!(self.width() <= width && self.height() <= height,
               "resizable buffers should only grow.");
        let mut o_new = Bitmap::new(width as usize, height as usize);
        o_new.patch(0, 0, self);
        *self = o_new;
    }
}

impl Font {
    pub fn render_char(&self, chr: char, scale: f32) -> Option<Bitmap> {
        use glyph_packer::Buffer2d;
        let info = match self.font.glyph(chr) {
            Some(a) => a,
            None => return None,
        };
        let info = info.scaled(rusttype::Pixels(scale));
        let info = info.positioned(rusttype::Point { x: 0.0, y:0.0 });
        let bb = match info.pixel_bounding_box() {
            Some(a) => a,
            None => return None
        };
        let mut out = Bitmap::new(bb.width() as usize, bb.height() as usize);
        info.draw(|x, y, v| {
            out.set(x, y, (v * 255.0) as u8);
        });
        Some(out)
    }

    pub fn make_atlas<I: Iterator<Item=char>>(&self, i: I, scale: f32, margin: u32, width: usize, height: usize) -> Bitmap {
        use glyph_packer::{Packer, GrowingPacker};
        let mut packer = glyph_packer::SkylinePacker::new(Bitmap::new(width, height));
        packer.set_margin(margin);
        for c in i.filter_map(|c| self.render_char(c, scale)) {
            packer.pack_resize(&c, |(ow, oh)| (ow * 2, oh * 2));
        }
        packer.into_buf()
    }
}
