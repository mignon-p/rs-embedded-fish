use byte_slice_cast::*;
use embedded_graphics::geometry::Point;
use embedded_graphics::geometry::Size;
use rand::Rng;

const FUDGE_FACTOR: i32 = 2;
const NUM_FRAMES: usize = 3;
const NUM_SPRITES: usize = 10;
const TRANSPARENT: u16 = 0xdead;

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
    frames: [&[u16]; NUM_FRAMES],
}

struct Fish {
    fish_type:  &Sprite,
    upper_left: Point,
    size:       Size,
    direction:  Dir,
    animation:  u8,
}

impl Sprite {
    fn get_point(&self, pt: &Point, animation: u8) -> PointValue {
        let x = pt.x - FUDGE_FACTOR;
        let y = pt.y - FUDGE_FACTOR;
        if (pt.x < 0 || pt.y < 0 ||
            pt.x >= self.size.width ||
            pt.y >= self.size.height) {
            Transparent
        } else {
            let idx = x + y * self.size.width;
            let c = frames[animation][idx];
            if c == TRANSPARENT {
                Transparent
            } else {
                Opaque(c)
            }
        }
    }

    fn make_sprite(sprite_num: usize, sprite_data: &[u16]) -> Sprite {
        let header_index = 4 * sprite_num;
        let width_height = sprite_data[header_index];
        let width = width_height >> 8;
        let height = width_height & 0xff;
        let num_words = width * height;

        let mut sprite = Sprite {
            size: Size::new(width, height),
            frames: [NUM_FRAMES: &[]],
        };

        for frame in (0..3) {
            let frame_index = sprite_data[header_index + frame + 1];
            sprite.frames[frame] =
                &sprite_data[frame_index..frame_index+num_words];
        }

        sprite
    }
}

impl Fish {
    fn get_point(&self, pt: &Point) -> PointValue {
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
            self.fish_type.get_point(Point::new(x, y), self.animation)
        }
    }

    fn on_screen(&self, screen: &Size) -> bool {
        (self.upper_left.y <= screen.height &&
         self.upper_left.y + self.size.height >= 0 &&
         self.upper_left.x <= screen.width &&
         self.upper_left.x + self.size.width >= 0)
    }

    fn randomize<T: Rng>(&mut self, rng: &mut T) {
        self.animation = rng.gen_range(1, NUM_FRAMES);
        self.direction = if rng.gen() { Dir::Left } else { Dir::Right };
        
    }
}
