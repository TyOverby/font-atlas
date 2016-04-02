#![deny(missing_docs)]

use std::collections::HashMap;
use super::rasterize::{Font, Bitmap, Atlas, CharInfo};
use super::glyph_packer;
use void::Void;

/// A cache that stores multiple fonts types of multiple weights.
///
/// The basic usage for a FontCache is to load all of the fonts that
/// you plan on using into the FontCache, then call `create_face` for
/// each of the font/size combinations that you see yourself using.
pub struct FontCache<T> {
    base_font: HashMap<String, Font>,
    cached: Vec<(String, f32, FaceCache<T>)>,
}

/// A cache for a single font/size pairing.
pub struct FaceCache<T> {
    font: Font,
    bitmap: T,
    scale: f32,
    atlas: Atlas,
    missing: HashMap<char, Option<T>>,
    missing_info: HashMap<char, CharInfo>
}

/// A command instructing the user on how to draw a
/// single character.
pub struct DrawCommand<'a, T: 'a> {
    /// The bitmap that contains the character
    pub bitmap: &'a T,
    /// The location of the character in the bitmap
    pub bitmap_location: glyph_packer::Rect,
    /// The location on the screen to draw the character
    pub draw_location: (f32, f32),
}

/// Drawing text is no easy business.  Here's a list of
/// everything that could go wrong.
#[derive(Debug)]
pub enum FontCacheError<E> {
    /// An error that comes from the bitmap transformation
    UserError(E),
    /// The font has not been loaded
    NoLoadedFont,
    /// A face has not been loaded and then rendered
    NoRenderedFace,
    /// A glyph is missing from the font file
    MissingGlyph,
}

impl <T> FontCache<T> {
    /// Create an empty FontCache.
    pub fn new() -> FontCache<T> {
        FontCache {
            base_font: HashMap::new(),
            cached: Vec::new(),
        }
    }

    /// Loads a font into the cache with a given name.  This will allow you to call
    /// `create_face` passing in the name in order to generate a rendering of this
    /// font at a given size.
    pub fn load_font<S: Into<String>>(&mut self, name: S, font: Font) {
        self.base_font.insert(name.into(), font);
    }

    /// Given the name of an already-loaded font and the scale at which to draw it,
    /// create_face generates an atlas with the characters in `chars` and stores it
    /// this cache.
    ///
    /// The function `f` is used to transform a character bitmap into whatever
    /// format you want to use internally.
    pub fn create_face<I, F, E>(&mut self, name: &str, scale: f32, chars: I, f: F) -> Result<(), FontCacheError<E>>
    where I: Iterator<Item=char>, F: Fn(Bitmap) -> Result<T, E> {
        if self.cached.iter().any(|&(ref n, s, _)| n == name && scale == s) {
            return Ok(());
        }

        match self.base_font.get(name).cloned() {
            Some(font) => {
                let fc = try!(FaceCache::new(font, scale, chars, f).or_else(|e| Err(FontCacheError::UserError(e))));
                self.cached.push((name.into(), scale, fc));
                return Ok(());
            }
            None => return Err(FontCacheError::NoLoadedFont)
        };
    }

    /// Retrieves a facecache reference given the name of the font and a scale (assuming one exists).
    pub fn get_face_cache(&self, name: &str, scale: f32) -> Option<&FaceCache<T>> {
        self.cached.iter()
                   .filter(|&&(ref n, s, _)| n == name && scale == s)
                   .map(|&(_, _, ref fc)| fc)
                   .next()
    }

    /// Retrieves a mutable facecache given the name of the font and a scale (assuming one exists).
    pub fn get_face_cache_mut(&mut self, name: &str, scale: f32) -> Option<&mut FaceCache<T>> {
        self.cached.iter_mut()
                   .filter(|&&mut (ref n, s, _)| n == name && scale == s)
                   .map(|&mut (_, _, ref mut fc)| fc)
                   .next()
    }

    /// Returns the drawing commands for a given font name, scale and the string to draw.
    ///
    /// Can fail if the font hasn't been rasterized with `create_face`.
    pub fn drawing_commands(&self, font_name: &str, scale: f32, string: &str) -> Result<Vec<DrawCommand<T>>, FontCacheError<Void>> {
        let fc = try!(self.get_face_cache(font_name, scale).ok_or(FontCacheError::NoRenderedFace));
        fc.drawing_commands(string)
    }

    /// Returns the drawing commands for a given font name, scale, and the string to draw.
    ///
    /// If the font hasn't been loaded with `load_font` an error is returned.
    ///
    /// If the font hasn't been rasterized with `create_face`, it is done so now.
    pub fn drawing_commands_prepared<F, E>(&mut self, font_name: &str, scale: f32, string: &str, f: F) -> Result<Vec<DrawCommand<T>>, FontCacheError<E>>
    where F: Fn(Bitmap) -> Result<T, E> {
        if self.get_face_cache(font_name, scale).is_none() {
            try!(self.create_face(font_name, scale, string.chars(), |a| f(a)));
        }
        {
            let fc = self.get_face_cache_mut(font_name, scale).unwrap();
            try!(fc.prepare_string(string, f).map_err(FontCacheError::UserError));
        }
        Ok(self.drawing_commands(font_name, scale, string).unwrap())
    }
}

impl <T> FaceCache<T> {
    /// Constructs a new FaceCache of a font at a specific scale.
    ///
    /// The characters in `chars` are pre-loaded into the atlas.
    /// If you need any more characters to draw things with, use
    /// `prepare_string`.
    pub fn new<I, F, E>(font: Font, scale: f32, chars: I, f: F) -> Result<FaceCache<T>, E>
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

    /// Adds all the characters in the given string to the cache.
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

    /// Returns true if a call to `prepare_string` is necessary to
    /// draw this string.
    pub fn needs_preparing(&self, s: &str) -> bool {
        for c in s.chars() {
            if self.atlas.info(c).is_none() && !self.missing.contains_key(&c) {
                return true;
            }
        }
        return false;
    }

    /// Returns a vector of drawing commands that describe how to lay out
    /// characters for printing
    pub fn drawing_commands(&self, s: &str) -> Result<Vec<DrawCommand<T>>, FontCacheError<Void>> {
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
                return Err(FontCacheError::MissingGlyph);
            }

            x += info.pre_draw_advance.0;
            y += info.pre_draw_advance.1;

            out.push(DrawCommand {
                bitmap: bitmap,
                bitmap_location: info.bounding_box,
                draw_location: (x, y),
            });
        }
        Ok(out)
    }

    /// Returns a Vec of DrawCommands that pre-prepare the face-cache for drawing.
    ///
    /// The only way that this will fail is if the rendering function `f` fails.
    pub fn drawing_commands_prepared<F, E>(&mut self, s: &str, f: F) -> Result<Vec<DrawCommand<T>>, E>
    where F: Fn(Bitmap) -> Result<T, E> {
        try!(self.prepare_string(s, f));
        Ok(self.drawing_commands(s).unwrap())
    }
}
