const FUDGE_FACTOR: i32 = 2;
const NUM_FRAMES: usize = 3;
const NUM_SPRITES: usize = 10;

const SPRITE_DATA: &[u8] = include_bytes!("fish.raw");

enum PointValue {
    OutOfRange,
    Transparent,
    Opaque(u16),
}

enum Dir {
    Left,
    Right,
}

struct Sprite {
    size: Size,
    frames: [&[u8]; NUM_FRAMES],
    color_map: &[u16],
}

struct Fish {
    fish_type:  &Sprite,
    upper_left: Point,
    size:       Size,
    direction:  Dir,
    animation:  u8,
}

impl Sprite {
    fn getPoint(&self, pt: &Point, animation: u8) -> PointValue {
        let x = pt.x - FUDGE_FACTOR;
        let y = pt.y - FUDGE_FACTOR;
        if (pt.x < 0 || pt.y < 0 ||
            pt.x >= self.size.width ||
            pt.y >= self.size.height) {
            Transparent
        } else {
            let idx = x + y * self.size.width;
            let c = frames[animation][idx];
            if c == 0 {
                Transparent
            } else {
                Opaque(self.color_map[c])
            }
        }
    }
}

impl Fish {
    fn getPoint(&self, pt: &Point) -> PointValue {
        if (pt.x < self.upper_left.x ||
            pt.y < self.upper_left.y ||
            pt.x >= self.upper_left.x + self.size.width ||
            pt.y >= self.upper_left.y + self.size.height) {
            OutOfRange
        } else {
            let mut x = pt.x - self.upper_left.x;
            let y = pt.y - self.upper_left.y;
            if self.direction == Dir::Left {
                x = self.size.width - (x + 1);
            }
            self.fish_type.getPoint(Point::new(x, y), self.animation)
        }
    }
}
