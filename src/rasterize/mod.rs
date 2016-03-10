use std::collections::HashMap;
use std::slice::Chunks;

use super::glyph_packer;
use super::rusttype;
use glyph_packer::{Packer, GrowingPacker};

pub struct Font {
    font: rusttype::Font<'static>
}

#[derive(Debug, Copy, Clone)]
pub struct CharInfo {
    pub chr: char,
    pub bounding_box: glyph_packer::Rect,
    pub post_draw_advance: (f32, f32),
    pub pre_draw_advance: (f32, f32),
}

pub struct Atlas {
    char_info: HashMap<char, CharInfo>
}

pub struct Bitmap {
    bytes: Vec<u8>,
    width: usize
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
    pub fn new(rusttype_font: rusttype::Font<'static>) -> Font {
        Font {
            font: rusttype_font
        }
    }

    pub fn render_char(&self, chr: char, scale: f32) -> Option<(CharInfo, Bitmap)> {
        use glyph_packer::Buffer2d;
        let glyph = match self.font.glyph(chr) {
            Some(a) => a,
            None => return None,
        };
        let glyph = glyph.scaled(rusttype::Pixels(scale));
        let glyph = glyph.positioned(rusttype::Point { x: 0.0, y:0.0 });
        let bb = match glyph.pixel_bounding_box() {
            Some(a) => a,
            None => return None
        };
        let mut out = Bitmap::new(bb.width() as usize, bb.height() as usize);
        glyph.draw(|x, y, v| {
            out.set(x, y, (v * 255.0) as u8);
        });

        let info = CharInfo {
            chr: chr,
            bounding_box: glyph_packer::Rect{
                x: bb.min.x as u32,
                y: bb.min.y as u32,
                w: bb.width() as u32,
                h: bb.height() as u32
            },
            post_draw_advance: (glyph.h_metrics().advance_width, 0.0),
            pre_draw_advance: (glyph.h_metrics().left_side_bearing, 0.0),
        };

        Some((info, out))
    }

    pub fn make_atlas<I: Iterator<Item=char>>(&self, i: I, scale: f32, margin: u32, width: usize, height: usize) -> (Atlas, Bitmap) {
        let mut atlas = Atlas { char_info: HashMap::new() };
        let mut packer = glyph_packer::SkylinePacker::new(Bitmap::new(width, height));
        packer.set_margin(margin);

        for c in i {
            if let Some((mut info, rendered)) = self.render_char(c, scale) {
                let r: glyph_packer::Rect = packer.pack_resize(&rendered, |(ow, oh)| (ow * 2, oh * 2));
                info.bounding_box = r;
                atlas.char_info.insert(c, info);
            }
        }
        (atlas, packer.into_buf())
    }
}

impl Atlas {
    pub fn info(&self, c: char) -> Option<CharInfo> {
        self.char_info.get(&c).cloned()
    }
}
