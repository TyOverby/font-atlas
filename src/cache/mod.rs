use std::collections::HashMap;
use super::rasterize::{Font, Bitmap, Atlas, CharInfo};
use super::glyph_packer;

pub struct FaceCache<T> {
    font: Font,
    bitmap: T,
    scale: f32,
    atlas: Atlas,
    missing: HashMap<char, Option<T>>,
    missing_info: HashMap<char, CharInfo>
}

pub struct DrawCommand<'a, T: 'a> {
    pub bitmap: &'a T,
    pub bitmap_location: glyph_packer::Rect,
    pub draw_location: (f32, f32),
}

impl <T> FaceCache<T> {
    pub fn new<I, F, E>(font: Font, chars: I, scale: f32, f: F) -> Result<FaceCache<T>, E>
    where I: Iterator<Item=char>, F: Fn(Bitmap) -> Result<T, E>
    {
            let (atlas, bitmap) = font.make_atlas(chars, scale, 3, 256, 256);
            let bitmap = try!(f(bitmap));
            Ok(FaceCache {
                font: font,
                atlas: atlas,
                bitmap: bitmap,
                scale: scale,
                missing: HashMap::new(),
                missing_info: HashMap::new(),
            })
    }

    pub fn prepare_string<F, E>(&mut self, s: &str, f: F) -> Result<(), E>
    where F: Fn(Bitmap) -> Result<T, E>
    {
        for c in s.chars() {
            if self.atlas.info(c).is_none() && !self.missing.contains_key(&c) {
                match self.font.render_char(c, self.scale).map(|(i, a)| (i, f(a))) {
                    Some((i, Ok(b))) => {
                        self.missing.insert(c, Some(b));
                        self.missing_info.insert(c, i);
                    },
                    Some((_, Err(e))) => return Err(e),
                    None => {
                        self.missing.insert(c, None);
                    },
                };
            }
        }
        Ok(())
    }

    pub fn drawing_commands_prepared<F, E>(&mut self, s: &str, f: F) -> Result<Vec<DrawCommand<T>>, E>
    where F: Fn(Bitmap) -> Result<T, E> {
        try!(self.prepare_string(s, f));
        Ok(self.drawing_commands(s))
    }

    pub fn drawing_commands(&self, s: &str) -> Vec<DrawCommand<T>> {
        let mut out = Vec::new();
        let mut x = 0.0;
        let mut y = 0.0;

        for c in s.chars() {
            let bitmap;
            let info;

            if let Some(ci) = self.atlas.info(c) {
                bitmap = &self.bitmap;
                info = ci;
            } else if let Some(ci) = self.missing_info.get(&c).cloned() {
                bitmap = self.missing.get(&c).unwrap().as_ref().unwrap();
                info = ci;
            } else {
                panic!("attempt to draw unprepared char {}", c);
            }

            x += info.pre_draw_advance.0;
            y += info.pre_draw_advance.1;

            out.push(DrawCommand {
                bitmap: bitmap,
                bitmap_location: info.bounding_box,
                draw_location: (x, y),
            });
        }
        out
    }
}
