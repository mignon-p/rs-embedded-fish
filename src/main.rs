use byte_slice_cast::*;
use embedded_graphics::drawable::Drawable;
use embedded_graphics::drawable::Pixel;
use embedded_graphics::geometry::Point;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::DrawTarget;
use rand::Rng;
use rand_pcg::Pcg32;

const FUDGE_FACTOR: i32 = 2;
const NUM_FRAMES: usize = 3;
const NUM_SPRITES: usize = 10;
const NUM_FISH: usize = 10;
const TRANSPARENT: u16 = 0xdead;
const BACKGROUND: u16 = 0x1f;   // blue

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

struct FishTank {
    sprites: [Sprite; NUM_SPRITES],
    fish:    [Fish;   NUM_FISH],
    size:    Size,
    rng:     Pcg32,
}

struct TankIterator {
    tank:     &FishTank,
    position: Point,
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

    fn randomize<T: Rng>(&mut self, screen: &Size, rng: &mut T) {
        self.animation = rng.gen_range(1, NUM_FRAMES);
        if rng.gen() {
            self.direction = Dir::Left;
            self.upper_left.x = screen.width;
        } else {
            self.direction = Dir::Right;
            self.upper_left.x = -self.size.width;
        }
        self.upper_left.y = rng.gen_range(0, screen.height - self.size.height);
    }

    fn randomize_x<T: Rng>(&mut self, screen: &Size, rng: &mut T) {
        self.upper_left.x = rng.gen_range(0, screen.width - self.size.width);
    }

    fn swim<T: Rng>(&mut self, screen: &Size, rng: &mut T) {
        if rng.gen() {
            self.upper_left.x += match self.direction {
                Dir::Left => -1,
                Dir::Right => 1,
            }
        }

        if rng.gen_ratio(1, 8) {
            self.upper_left.y += rng.gen_range(-1, 2);
        }

        self.animation += 1;
        if self.animation >= NUM_FRAMES {
            self.animation = 0;
        }

        if self.on_screen(screen) == false {
            self.randomize(screen. rng);
        }
    }

    fn new(sprite: &Sprite) -> Fish {
        let ff2 = FUDGE_FACTOR * 2;
        Fish {
            fish_type:  sprite,
            upper_left: Point::new(0, 0),
            size:       Size::new(sprite.size.width + ff2,
                                  sprite.size.height + ff2),
            direction:  Dir::Right,
            animation:  0,
        }
    }
}

impl FishTank {
    fn new(screen_size: Size, seed: u64) -> FishTank {
        let sprite_data = SPRITE_DATA.as_slice_of::<u16>().unwrap();
        let dummy_sprite = Sprite::make_sprite(0, sprite_data);
        let mut tank = FishTank {
            sprites: [dummy_sprite; NUM_SPRITES],
            fish:    [Fish::new(&dummy_sprite); NUM_FISH],
            size:    screen_size,
            rng:     Pcg32::new(seed),
        };

        for i in (0..NUM_FISH) { // assumes NUM_FISH <= NUM_SPRITES
            tank.sprites[i] = Sprite::make_sprite(i, sprite_data);
            tank.fish[i]    = Fish::new(&tank.sprites[i]);
            tank.fish[i].randomize  (tank.size, tank.rng);
            tank.fish[i].randomize_x(tank.size, tank.rng);
        }

        tank
    }

    fn swim(&mut self) {
        for i in (0..NUM_FISH) {
            self.fish[i].swim(&self.size, &mut self.rng);
        }
    }

    fn get_point(&self, pt: &Point) -> PointValue {
        let mut ret = OutOfRange;
        for i in (0..NUM_FISH) {
            match self.fish[i].get_point(pt) {
                Opaque(c)   => return Opaque(c),
                Transparent => ret = Transparent,
                OutOfRange  => (),
            }
        }

        ret
    }
}

impl IntoIterator for FishTank {
    type Item = Pixel<Rgb565>;
    type IntoIter = TankIterator;

    fn into_iter(self) -> Self::IntoIter {
        TankIterator::new(self)
    }
}

impl Drawable<Rgb565> for FishTank {
    fn draw<D: DrawTarget<Rgb565>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(self)
    }
}

impl TankIterator {
    fn new(fish_tank: &FishTank) -> TankIterator {
        TankIterator {
            tank:     fish_tank,
            position: Point::new(0, 0),
        }
    }

    fn some_color(c: u16) -> Option<Self::Item> {
        Pixel(self.position, RawU16::from_u32(c))
    }
}

impl Iterator for TankIterator {
    type Item = Pixel<Rgb565>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.position.y >= tank.size.height {
                return None;
            } else {
                let pv = tank.get_point(&self.position);
                let ret = match pv {
                    OutOfRange    => None,
                    Transparent   => some_color(BACKGROUND),
                    Opaque(color) => some_color(color),
                }

                self.position.x += 1;
                if self.position.x >= tank.size.width {
                    self.position.x = 0;
                    self.position.y += 1;
                }

                if let Some(_) = ret {
                    return ret;
                }
            }
        }
    }
}
