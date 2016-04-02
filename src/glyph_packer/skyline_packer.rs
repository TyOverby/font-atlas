use std::cmp::max;

use super::{
    Buffer2d,
    Rect,
    Packer,
};

struct Skyline {
    pub x: u32,
    pub y: u32,
    pub w: u32,
}

/// Contains the state of the skyline packing process
pub struct SkylinePacker<B: Buffer2d> {
    buf: B,
    width: u32,
    height: u32,
    skylines: Vec<Skyline>,
    margin: u32,
}

impl<B: Buffer2d> SkylinePacker<B> {

    fn can_put(&self, i: usize, w: u32, h: u32) -> Option<u32> {
        let x = self.skylines[i].x;
        if x + w > self.width {
            return None;
        }
        let mut width_left = w;
        let mut i = i;
        let mut y = self.skylines[i].y;
        loop {
            y = max(y, self.skylines[i].y);
            if y + h > self.height {
                return None;
            }
            if self.skylines[i].w > width_left {
                return Some(y);
            }
            width_left -= self.skylines[i].w;
            i += 1;
            if i >= self.skylines.len() {
                return None;
            }
        }
    }

    fn find_skyline(&self, w: u32, h: u32) -> Option<(usize, Rect)> {
        let mut min_height = ::std::u32::MAX;
        let mut min_width = ::std::u32::MAX;
        let mut index = None;
        let mut rect = Rect::new(0, 0, 0, 0);

        // keep the min_height as small as possible
        for i in 0 .. self.skylines.len() {
            if let Some(y) = self.can_put(i, w, h) {
                if y + h < min_height ||
                    (y + h == min_height && self.skylines[i].w < min_width) {
                        min_height = y + h;
                        min_width = self.skylines[i].w;
                        index = Some(i);
                        rect.x = self.skylines[i].x;
                        rect.y = y;
                        rect.w = w;
                        rect.h = h;
                    }
            }

            /*
            if let Some(y) = self.can_put(i, h, w) {
                if y + w < min_height ||
                    (y + w == min_height && self.skylines[i].w < min_width) {
                        min_height = y + w;
                        min_width = self.skylines[i].w;
                        index = Some(i);
                        rect.x = self.skylines[i].x;
                        rect.y = y;
                        rect.w = h;
                        rect.h = w;
                    }
            }*/
        }

        if index.is_some() {
            Some((index.unwrap(), rect))
        } else {
            None
        }
    }

    fn split(&mut self, index: usize, rect: &Rect) {
        let skyline = Skyline {
            x: rect.x,
            y: rect.y + rect.h,
            w: rect.w,
        };

        assert!(skyline.x + skyline.w <= self.width);
        assert!(skyline.y <= self.height);

        self.skylines.insert(index, skyline);

        let i = index + 1;
        while i < self.skylines.len() {
            assert!(self.skylines[i-1].x <= self.skylines[i].x);

            if self.skylines[i].x < self.skylines[i-1].x + self.skylines[i-1].w {
                let shrink = self.skylines[i-1].x + self.skylines[i-1].w - self.skylines[i].x;
                if self.skylines[i].w <= shrink {
                    self.skylines.remove(i);
                } else {
                    self.skylines[i].x += shrink;
                    self.skylines[i].w -= shrink;
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn merge(&mut self) {
        let mut i = 1;
        while i < self.skylines.len() {
            if self.skylines[i-1].y == self.skylines[i].y {
                self.skylines[i-1].w += self.skylines[i].w;
                self.skylines.remove(i);
                i -= 1;
            }
            i += 1;
        }
    }
}

impl<B: Buffer2d> Packer for SkylinePacker<B> {
    type Buffer = B;

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_dimensions(&mut self, w: u32, h: u32) {
        let (old_w, _) = self.dimensions();

        self.width = w;
        self.height = h;
        self.skylines.push(Skyline {
            x: old_w,
            y: 0,
            w: w - old_w
        });
    }

    fn new(buf: B) -> SkylinePacker<B> {
        let (width, height) = buf.dimensions();
        let mut skylines = Vec::new();
        skylines.push(Skyline {
            x: 0,
            y: 0,
            w: width,
        });

        SkylinePacker {
            buf: buf,
            width: width,
            height: height,
            skylines: skylines,
            margin: 0,
        }
    }

    fn pack<O: Buffer2d<Pixel=B::Pixel>>(&mut self, buf: &O) -> Option<Rect> {
        let (mut width, mut height) = buf.dimensions();
        width += self.margin;
        height += self.margin;

        if let Some((i, mut rect)) = self.find_skyline(width, height) {
            if width == rect.w {
                self.buf.patch(rect.x, rect.y, buf);
            } else {
                self.buf.patch_rotated(rect.x, rect.y, buf);
            }

            self.split(i, &rect);
            self.merge();

            rect.w -= self.margin;
            rect.h -= self.margin;
            Some(rect)
        } else { None }
    }

    fn buf(&self) -> &B {
        &self.buf
    }

    fn buf_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    fn into_buf(self) -> B {
        self.buf
    }

    fn set_margin(&mut self, val: u32) {
        self.margin = val;
    }
}

