use std::collections::HashMap;
use std::boxed::Box;

use super::rasterize::{Font, Bitmap};


struct FaceCache<T, E> {
    font: Font,
    atlas: T,
    missing: HashMap<char, T>,
    trx: Box<Fn(Bitmap) -> Result<T, E>>,
}

impl <T, E> FaceCache<T, E> {
    fn new<I, F>(font: Font, chars: I, f: F) -> Result<FaceCache<T, E>, E>
    where I: Iterator<Item=char>, F: Fn(Bitmap) -> Result<T, E> + 'static
    {
            let atlas_bm = font.make_atlas(chars, 20.0, 1, 256, 256);
            let atlas = try!(f(atlas_bm));
            Ok(FaceCache {
                font: font,
                atlas: atlas,
                missing: HashMap::new(),
                trx: Box::new(f)
            })
    }
}
