/// A rectangle
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    /// Create a new rectangle given the top-left corner and
    /// width + height
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }

    /// Create a new rectangle from a pair of points
    pub fn new_with_points(x1: u32, y1: u32, x2: u32, y2: u32) -> Rect {
        Rect {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1,
        }
    }

    /// Return the position of the top edge of the rectangle
    #[inline(always)]
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Return the position of the bottom edge of the rectangle
    #[inline(always)]
    pub fn bottom(&self) -> u32 {
        self.y + self.h
    }

    /// Return the position of the left edge of the rectangle
    #[inline(always)]
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Return the position of the right edge of the rectangle
    #[inline(always)]
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    /// Return the area of the rectangle
    #[inline(always)]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    /// Returns true if this rectangle intersects another rectangle
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right() &&
        self.right() > other.left() &&
        self.top() < other.bottom() &&
        self.bottom() > other.top()
    }

    /// Returns true if this rectangle completely contains another rectangle
    pub fn contains(&self, other: &Rect) -> bool {
        self.left() <= other.left() &&
        self.right() >= other.right() &&
        self.top() <= other.top() &&
        self.bottom() >= other.bottom()
    }

    /// Returns true if this rectangle completely contains another rectangle
    pub fn crop(&self, other: &Rect) -> Vec<Rect> {
        if !self.intersects(other) {
            return vec!(*self);
        }

        let inside_x1 = if other.left() < self.left() {
            self.left()
        } else {
            other.left()
        };

        let inside_y1 = if other.top() < self.top() {
            self.top()
        } else {
            other.top()
        };

        let inside_x2 = if other.right() > self.right() {
            self.right()
        } else {
            other.right()
        };

        let inside_y2 = if other.bottom() > self.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };

        //
        // *******************
        // *    | r3  |      *
        // *    |     |      *
        // *    +++++++      *
        // * r1 +     +      *
        // *    +     +  r2  *
        // *    +++++++      *
        // *    |     |      *
        // *    | r4  |      *
        // *******************
        //
        let mut result = Vec::new();

        let r1 = Rect::new_with_points(self.left(), self.top(), inside_x1, self.bottom());
        if r1.area() > 0 {
            result.push(r1);
        }

        let r2 = Rect::new_with_points(inside_x2, self.top(), self.right(), self.bottom());
        if r2.area() > 0 {
            result.push(r2);
        }

        let r3 = Rect::new_with_points(inside_x1, self.top(), inside_x2, inside_y1);
        if r3.area() > 0 {
            result.push(r3);
        }

        let r4 = Rect::new_with_points(inside_x1, inside_y2, inside_x2, self.bottom());
        if r4.area() > 0 {
            result.push(r4);
        }

        result
    }
}

