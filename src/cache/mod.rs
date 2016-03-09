use std::collections::HashMap;
use super::rasterize::{Font, Bitmap};

pub struct FaceCache<T> {
    font: Font,
    atlas: T,
    missing: HashMap<char, T>,
}

impl <T> FaceCache<T> {
    fn new<I, F, E>(font: Font, chars: I, f: F) -> Result<FaceCache<T>, E>
    where I: Iterator<Item=char>, F: Fn(Bitmap) -> Result<T, E>
    {
            let atlas_bm = font.make_atlas(chars, 20.0, 1, 256, 256);
            let atlas = try!(f(atlas_bm));
            Ok(FaceCache {
                font: font,
                atlas: atlas,
                missing: HashMap::new(),
            })
    }
}
