use super::{Buffer2d, ResizeBuffer, Rect};

pub trait Packer {
    type Buffer: Buffer2d;

    fn new(b: Self::Buffer) -> Self;
    fn pack<O: Buffer2d<Pixel=<Self::Buffer as Buffer2d>::Pixel>>(&mut self, buf: &O) -> Option<Rect>;
    fn set_margin(&mut self, _val: u32) {}
    fn buf(&self) -> &Self::Buffer;
    fn buf_mut(&mut self) -> &mut Self::Buffer;
    fn into_buf(self) -> Self::Buffer;

    fn dimensions(&self) -> (u32, u32);
    fn set_dimensions(&mut self, w: u32, h: u32);
}

pub trait GrowingPacker: Packer {
    fn pack_resize<F, O>(&mut self, buf: &O, resize_fn: F) -> Rect
    where O: Buffer2d<Pixel=<<Self as Packer>::Buffer as Buffer2d>:: Pixel>,
          F: Fn((u32, u32)) -> (u32, u32);
}

impl <A, P: Packer<Buffer = A>> GrowingPacker for P where A: ResizeBuffer {
    fn pack_resize<F, O>(&mut self, buf: &O, resize_fn: F) -> Rect
    where O: Buffer2d<Pixel=<<Self as Packer>::Buffer as Buffer2d>:: Pixel>,
          F: Fn((u32, u32)) -> (u32, u32) {
        match self.pack(buf) {
            Some(p) => p,
            None => {
                let (w, h) = self.dimensions();
                let (nw, nh) = resize_fn((w, h));
                if nw <= w || nh <= h {
                    panic!("Resize function must make the buffer larger");
                }
                self.buf_mut().resize(nw, nh);
                self.set_dimensions(nw, nh);
                self.pack_resize(buf, resize_fn)
            }
        }
    }
}
