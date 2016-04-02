#![deny(missing_docs)]

use std::collections::HashMap;
use std::slice::Chunks;

use super::glyph_packer;
use super::rusttype;
use glyph_packer::{Packer, GrowingPacker};

/// A single font loaded from a file.
#[derive(Clone)]
pub struct Font {
    font: rusttype::Font<'static>
}

/// Information about a character from a font rendered
/// at a specific scale.
#[derive(Debug, Copy, Clone)]
pub struct CharInfo {
    /// The character in question
    pub chr: char,
    /// The scale that the character was rendered at
    pub scale: f32,
    /// The size of the character
    pub bounding_box: glyph_packer::Rect,
    /// The amount of (x, y) that the pen should move
    /// after drawing the character
    pub post_draw_advance: (f32, f32),
    /// The amount of (x, y) that the pen should move
    /// before drawing the character
    pub pre_draw_advance: (f32, f32),
}

/// A mapping from chars to CharInfo.
pub struct Atlas {
    char_info: HashMap<char, CharInfo>
}

/// A rectangular 2d-array of u8 where
/// the values 0 through 255 represent
/// shades of grey.
pub struct Bitmap {
    bytes: Vec<u8>,
    width: usize
}

impl Bitmap {
    /// Construct a new empty (all zeros) bitmap
    /// of the given dimensions.
    fn new(w: usize, h: usize) -> Bitmap {
        Bitmap {
            bytes: vec![0; w * h],
            width: w,
        }
    }

    /// Return an iterator over the lines of the bitmap
    /// going from top to bottom.
    pub fn lines(&self) -> Chunks<u8> {
        self.bytes.chunks(self.width)
    }

    /// The width of this bitmap
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height of this bitmap
    pub fn height(&self) -> usize {
        self.bytes.len() / self.width()
    }

    /// Gain access to the underlying slice of u8.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the underlying buffer of u8
    pub fn into_raw(self) -> Vec<u8> {
        self.bytes
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
        let x = x as usize;
        let y = y as usize;
        let width = self.width() as usize();
        self.bytes.get(x + width * y).cloned()
    }

    fn set(&mut self, x: u32, y: u32, val: Self::Pixel) {
        let x = x as usize;
        let y = y as usize;
        let width = self.width() as usize();
        if let Some(p) = self.bytes.get_mut(x + width * y) {
            *p = val;
        }
    }
}

impl glyph_packer::ResizeBuffer for Bitmap {
    fn resize(&mut self, width: u32, height: u32) {
        use glyph_packer::Buffer2d;
        assert!(self.width() <= width as usize && self.height() <= height as usize,
               "resizable buffers should only grow.");
        let mut o_new = Bitmap::new(width as usize, height as usize);
        o_new.patch(0, 0, self);
        *self = o_new;
    }
}

impl Font {
    /// Construct a new Font from a rusttype::Font.
    pub fn new(rusttype_font: rusttype::Font<'static>) -> Font {
        Font {
            font: rusttype_font
        }
    }

    /// Renders a character from this font at a given scale into a pair of (CharInfo, Bitmap).
    ///
    /// If the character isn't handled by the font, None is returned.
    pub fn render_char(&self, chr: char, scale: f32) -> Option<(CharInfo, Bitmap)> {
        use glyph_packer::Buffer2d;
        let glyph = match self.font.glyph(chr) {
            Some(a) => a,
            None => return None,
        };
        let glyph = glyph.scaled(rusttype::Scale::uniform(scale));
        let h_metrics = glyph.h_metrics();
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
            scale: scale,
            bounding_box: glyph_packer::Rect{
                x: bb.min.x as u32,
                y: bb.min.y as u32,
                w: bb.width() as u32,
                h: bb.height() as u32
            },
            post_draw_advance: (h_metrics.advance_width, 0.0),
            pre_draw_advance: (h_metrics.left_side_bearing, 0.0),
        };

        Some((info, out))
    }

    /// Creates an atlas for a set of characters rendered at a given scale.
    ///
    /// `margin` is the distance between characters in pixels.
    /// `width` and `height` denote the starting size of the bitmap.
    ///
    /// The resulting bitmap may be larger than width x height in order to
    /// fit all of the characters.
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
    /// Returns the information about a rendered character if one exists
    pub fn info(&self, c: char) -> Option<CharInfo> {
        self.char_info.get(&c).cloned()
    }
}
