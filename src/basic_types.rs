// basic_types.rs
// Basic types used throughout the project: Compass, Dir, Coord, Polar

use std::f32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Compass {
    SW = 0,
    S = 1,
    SE = 2,
    W = 3,
    Center = 4,
    E = 5,
    NW = 6,
    N = 7,
    NE = 8,
}

// Supports the eight directions in enum Compass plus CENTER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Dir {
    dir9: Compass,
}

impl Dir {
    pub fn random8() -> Self {
        use crate::random::random_uint_range;
        Dir {
            dir9: match random_uint_range(0, 7) {
                0 => Compass::SW,
                1 => Compass::S,
                2 => Compass::SE,
                3 => Compass::W,
                4 => Compass::E,
                5 => Compass::NW,
                6 => Compass::N,
                7 => Compass::NE,
                _ => Compass::Center,
            },
        }
    }

    pub fn new(dir: Compass) -> Self {
        Dir { dir9: dir }
    }

    pub fn as_int(&self) -> u8 {
        self.dir9 as u8
    }

    pub fn as_normalized_coord(&self) -> Coord {
        NORMALIZED_COORDS[self.as_int() as usize]
    }

    pub fn as_normalized_polar(&self) -> Polar {
        Polar {
            mag: 1,
            dir: *self,
        }
    }

    pub fn rotate(&self, n: i32) -> Self {
        let idx = (self.as_int() as usize * 8) + ((n & 7) as usize);
        Dir {
            dir9: ROTATIONS[idx],
        }
    }

    pub fn rotate90_deg_cw(&self) -> Self {
        self.rotate(2)
    }

    pub fn rotate90_deg_ccw(&self) -> Self {
        self.rotate(-2)
    }

    pub fn rotate180_deg(&self) -> Self {
        self.rotate(4)
    }
}

impl Default for Dir {
    fn default() -> Self {
        Dir {
            dir9: Compass::Center,
        }
    }
}

impl From<Compass> for Dir {
    fn from(dir: Compass) -> Self {
        Dir { dir9: dir }
    }
}

// Coordinates range anywhere in the range of i16. Coordinate arithmetic
// wraps like i16. Can be used, e.g., for a location in the simulator grid, or
// for the difference between two locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Coord {
    pub x: i16,
    pub y: i16,
}

impl Coord {
    pub fn new(x: i16, y: i16) -> Self {
        Coord { x, y }
    }

    pub fn is_normalized(&self) -> bool {
        self.x >= -1 && self.x <= 1 && self.y >= -1 && self.y <= 1
    }

    pub fn normalize(&self) -> Self {
        self.as_dir().as_normalized_coord()
    }

    pub fn length(&self) -> u32 {
        ((self.x as f64 * self.x as f64 + self.y as f64 * self.y as f64).sqrt()) as u32
    }

    pub fn as_dir(&self) -> Dir {
        // tanN/tanD is the best rational approximation to tan(22.5) under the constraint that
        // tanN + tanD < 2**16 (to avoid overflows).
        const TAN_N: u16 = 13860;
        const TAN_D: u16 = 33461;
        const CONVERSION: [Compass; 16] = [
            Compass::S,
            Compass::Center,
            Compass::SW,
            Compass::N,
            Compass::SE,
            Compass::E,
            Compass::N,
            Compass::N,
            Compass::N,
            Compass::N,
            Compass::W,
            Compass::NW,
            Compass::N,
            Compass::NE,
            Compass::N,
            Compass::N,
        ];

        let xp = (self.x as i32) * (TAN_D as i32) + (self.y as i32) * (TAN_N as i32);
        let yp = (self.y as i32) * (TAN_D as i32) - (self.x as i32) * (TAN_N as i32);

        let idx = ((yp > 0) as usize * 8)
            + ((xp > 0) as usize * 4)
            + ((yp > xp) as usize * 2)
            + ((yp >= -xp) as usize);
        Dir {
            dir9: CONVERSION[idx],
        }
    }

    pub fn as_polar(&self) -> Polar {
        Polar {
            mag: self.length() as i32,
            dir: self.as_dir(),
        }
    }

    // returns -1.0 (opposite directions) .. +1.0 (same direction)
    // returns 1.0 if either vector is (0,0)
    pub fn ray_sameness(&self, other: Coord) -> f32 {
        let mag = ((self.x as i64 * self.x as i64 + self.y as i64 * self.y as i64)
            * (other.x as i64 * other.x as i64 + other.y as i64 * other.y as i64)) as f64;
        if mag == 0.0 {
            return 1.0; // anything is "same" as zero vector
        }
        ((self.x as i64 * other.x as i64 + self.y as i64 * other.y as i64) as f64 / mag.sqrt())
            as f32
    }

    pub fn ray_sameness_dir(&self, d: Dir) -> f32 {
        self.ray_sameness(d.as_normalized_coord())
    }
}

impl Default for Coord {
    fn default() -> Self {
        Coord { x: 0, y: 0 }
    }
}

impl std::ops::Add<Coord> for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord {
            x: (self.x + other.x) as i16,
            y: (self.y + other.y) as i16,
        }
    }
}

impl std::ops::Sub<Coord> for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord {
            x: (self.x - other.x) as i16,
            y: (self.y - other.y) as i16,
        }
    }
}

impl std::ops::Mul<i32> for Coord {
    type Output = Coord;
    fn mul(self, a: i32) -> Coord {
        Coord {
            x: (self.x as i32 * a) as i16,
            y: (self.y as i32 * a) as i16,
        }
    }
}

impl std::ops::Add<Dir> for Coord {
    type Output = Coord;
    fn add(self, d: Dir) -> Coord {
        self + d.as_normalized_coord()
    }
}

impl std::ops::Sub<Dir> for Coord {
    type Output = Coord;
    fn sub(self, d: Dir) -> Coord {
        self - d.as_normalized_coord()
    }
}

// Polar magnitudes are signed 32-bit integers so that they can extend across any 2D
// area defined by the Coord class.
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Polar {
    pub mag: i32,
    pub dir: Dir,
}

impl Polar {
    pub fn new(mag: i32, dir: Compass) -> Self {
        Polar {
            mag,
            dir: Dir::new(dir),
        }
    }

    pub fn new_with_dir(mag: i32, dir: Dir) -> Self {
        Polar { mag, dir }
    }

    pub fn as_coord(&self) -> Coord {
        // 3037000500 is 1/sqrt(2) in 32.32 fixed point
        const COORD_MAGS: [i64; 9] = [
            3037000500, // SW
            1i64 << 32, // S
            3037000500, // SE
            1i64 << 32, // W
            0,          // CENTER
            1i64 << 32, // E
            3037000500, // NW
            1i64 << 32, // N
            3037000500, // NE
        ];

        let len = COORD_MAGS[self.dir.as_int() as usize] * (self.mag as i64);

        let temp = ((self.mag as i64) >> 32) ^ ((1i64 << 31) - 1);
        let len = (len + temp) / (1i64 << 32);

        NORMALIZED_COORDS[self.dir.as_int() as usize] * (len as i32)
    }
}

impl From<Coord> for Polar {
    fn from(coord: Coord) -> Self {
        coord.as_polar()
    }
}

// Constants
const NORMALIZED_COORDS: [Coord; 9] = [
    Coord { x: -1, y: -1 }, // SW
    Coord { x: 0, y: -1 },  // S
    Coord { x: 1, y: -1 },  // SE
    Coord { x: -1, y: 0 },  // W
    Coord { x: 0, y: 0 },   // CENTER
    Coord { x: 1, y: 0 },   // E
    Coord { x: -1, y: 1 },  // NW
    Coord { x: 0, y: 1 },   // N
    Coord { x: 1, y: 1 },   // NE
];

const ROTATIONS: [Compass; 72] = [
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::W,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::Center,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::NW,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::N,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::NE,
    Compass::E,
    Compass::SE,
    Compass::S,
    Compass::SW,
    Compass::W,
    Compass::NW,
    Compass::N,
];

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function: returns true only if a and b are the same to within 4 digits accuracy
    fn are_close_f(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.0001
    }

    // Helper function to safely extract Coord fields from packed struct
    fn coord_fields(c: Coord) -> (i16, i16) {
        (c.x, c.y)
    }

    // Helper function to safely extract Polar fields from packed struct
    fn polar_fields(p: Polar) -> (i32, Compass) {
        (p.mag, p.dir.dir9)
    }

    // Helper function to safely extract Dir field from packed struct
    fn dir_field(d: Dir) -> Compass {
        d.dir9
    }

    #[test]
    fn test_dir_basic_types() {
        // Dir ctor from Compass
        let mut d1;
        let mut d2 = Dir::new(Compass::Center);
        d1 = d2;
        assert_eq!(d1.as_int(), Compass::Center as u8);
        d1 = Dir::new(Compass::SW);
        assert_eq!(d1.as_int(), 0);
        d1 = Dir::new(Compass::S);
        assert_eq!(d1.as_int(), 1);
        d1 = Dir::new(Compass::SE);
        assert_eq!(d1.as_int(), 2);
        d1 = Dir::new(Compass::W);
        assert_eq!(d1.as_int(), 3);
        d1 = Dir::new(Compass::Center);
        assert_eq!(d1.as_int(), 4);
        d1 = Dir::new(Compass::E);
        assert_eq!(d1.as_int(), 5);
        d1 = Dir::new(Compass::NW);
        assert_eq!(d1.as_int(), 6);
        d1 = Dir::new(Compass::N);
        assert_eq!(d1.as_int(), 7);
        d1 = Dir::new(Compass::NE);
        assert_eq!(d1.as_int(), 8);

        assert_eq!(Dir::new(Compass::SW).as_int(), 0);
        assert_eq!(Dir::new(Compass::S).as_int(), 1);
        assert_eq!(Dir::new(Compass::SE).as_int(), 2);
        assert_eq!(Dir::new(Compass::W).as_int(), 3);
        assert_eq!(Dir::new(Compass::Center).as_int(), 4);
        assert_eq!(Dir::new(Compass::E).as_int(), 5);
        assert_eq!(Dir::new(Compass::NW).as_int(), 6);
        assert_eq!(Dir::new(Compass::N).as_int(), 7);
        assert_eq!(Dir::new(Compass::NE).as_int(), 8);
        assert_eq!(Dir::new(unsafe { std::mem::transmute(8u8) }).as_int(), 8);
        
        d2 = Dir::from(Compass::E);
        d1 = d2;
        assert_eq!(d1.as_int(), 5);
        d2 = d1;
        assert_eq!(d1.as_int(), 5);
        assert_eq!(d2.as_int(), 5); // Verify d2 was set correctly

        // operator= from Compass
        d1 = Dir::from(Compass::SW);
        assert_eq!(d1.as_int(), 0);
        d1 = Dir::from(Compass::SE);
        assert_eq!(d1.as_int(), 2);

        // [in]equality with Compass
        d1 = Dir::from(Compass::Center);
        assert_eq!(d1.dir9, Compass::Center);
        d1 = Dir::from(Compass::SE);
        assert_eq!(d1.dir9, Compass::SE);
        assert_eq!(Dir::new(Compass::W).dir9, Compass::W);
        assert_ne!(Dir::new(Compass::W).dir9, Compass::NW);

        // [in]equality with Dir
        d1 = Dir::from(Compass::N);
        d2 = Dir::from(Compass::N);
        assert_eq!(d1, d2);
        assert_eq!(d2, d1);
        d1 = Dir::from(Compass::NE);
        assert_ne!(d1, d2);
        assert_ne!(d2, d1);

        // .rotate()
        d1 = Dir::from(Compass::NE);
        assert_eq!(d1.rotate(1).dir9, Compass::E);
        assert_eq!(d1.rotate(2).dir9, Compass::SE);
        assert_eq!(d1.rotate(-1).dir9, Compass::N);
        assert_eq!(d1.rotate(-2).dir9, Compass::NW);
        assert_eq!(Dir::new(Compass::N).rotate(1), d1);
        assert_eq!(Dir::new(Compass::SW).rotate(-2).dir9, Compass::SE);

        // .asNormalizedCoord()
        let mut c1 = Dir::new(Compass::Center).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        d1 = Dir::from(Compass::SW);
        c1 = d1.as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, -1);
        c1 = Dir::new(Compass::S).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, -1);
        c1 = Dir::new(Compass::SE).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, -1);
        c1 = Dir::new(Compass::W).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, 0);
        c1 = Dir::new(Compass::E).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        c1 = Dir::new(Compass::NW).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, 1);
        c1 = Dir::new(Compass::N).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 1);
        c1 = Dir::new(Compass::NE).as_normalized_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 1);

        // .asNormalizedPolar()
        d1 = Dir::from(Compass::SW);
        let mut p1 = d1.as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::SW);
        p1 = Dir::new(Compass::S).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::S);
        p1 = Dir::new(Compass::SE).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::SE);
        p1 = Dir::new(Compass::W).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::W);
        p1 = Dir::new(Compass::E).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::E);
        p1 = Dir::new(Compass::NW).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::NW);
        p1 = Dir::new(Compass::N).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::N);
        p1 = Dir::new(Compass::NE).as_normalized_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::NE);
    }

    #[test]
    fn test_coord_basic_types() {
        // Coord ctor from i16, i16
        let mut c1 = Coord::default();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c1 = Coord::new(1, 1);
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        c1 = Coord::new(-6, 12);
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -6);
        assert_eq!(y, 12);

        // copy assignment
        let c2 = Coord::new(9, 101);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 9);
        assert_eq!(y, 101);
        c1 = c2;
        let (x, _y) = coord_fields(c1);
        assert_eq!(x, 9);
        let (_x2, y2) = coord_fields(c2);
        assert_eq!(y2, 101);

        // .isNormalized()
        assert!(!c1.is_normalized());
        assert!(Coord::new(0, 0).is_normalized());
        assert!(Coord::new(0, 1).is_normalized());
        assert!(Coord::new(1, 1).is_normalized());
        assert!(Coord::new(-1, 0).is_normalized());
        assert!(Coord::new(-1, -1).is_normalized());
        assert!(!Coord::new(0, 2).is_normalized());
        assert!(!Coord::new(1, 2).is_normalized());
        assert!(!Coord::new(-1, 2).is_normalized());
        assert!(!Coord::new(-2, 0).is_normalized());

        // .normalize() and .asDir()
        c1 = Coord::new(0, 0);
        let c2 = c1.normalize();
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(dir_field(c2.as_dir()), Compass::Center);
        c1 = Coord::new(0, 1).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 1);
        assert_eq!(dir_field(c1.as_dir()), Compass::N);
        c1 = Coord::new(-1, 1).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, 1);
        assert_eq!(dir_field(c1.as_dir()), Compass::NW);
        c1 = Coord::new(100, 5).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        assert_eq!(dir_field(c1.as_dir()), Compass::E);
        c1 = Coord::new(100, 105).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        assert_eq!(dir_field(c1.as_dir()), Compass::NE);
        c1 = Coord::new(-5, 101).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 1);
        assert_eq!(dir_field(c1.as_dir()), Compass::N);
        c1 = Coord::new(-500, 10).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, 0);
        assert_eq!(dir_field(c1.as_dir()), Compass::W);
        c1 = Coord::new(-500, -490).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -1);
        assert_eq!(y, -1);
        assert_eq!(dir_field(c1.as_dir()), Compass::SW);
        c1 = Coord::new(-1, -490).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, -1);
        assert_eq!(dir_field(c1.as_dir()), Compass::S);
        c1 = Coord::new(1101, -1090).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, -1);
        assert_eq!(dir_field(c1.as_dir()), Compass::SE);
        c1 = Coord::new(1101, -3).normalize();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        assert_eq!(dir_field(c1.as_dir()), Compass::E);

        // .length()
        assert_eq!(Coord::new(0, 0).length(), 0);
        assert_eq!(Coord::new(0, 1).length(), 1);
        assert_eq!(Coord::new(-1, 0).length(), 1);
        assert_eq!(Coord::new(-1, -1).length(), 1); // round down
        assert_eq!(Coord::new(22, 0).length(), 22);
        assert_eq!(Coord::new(22, 22).length(), 31); // round down
        assert_eq!(Coord::new(10, -10).length(), 14); // round down
        assert_eq!(Coord::new(-310, 0).length(), 310);

        // .asPolar()
        let mut p1 = Coord::new(0, 0).as_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 0);
        assert_eq!(dir, Compass::Center);
        p1 = Coord::new(0, 1).as_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 1);
        assert_eq!(dir, Compass::N);
        p1 = Coord::new(-10, -10).as_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 14);
        assert_eq!(dir, Compass::SW); // round down mag
        p1 = Coord::new(100, 1).as_polar();
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 100);
        assert_eq!(dir, Compass::E); // round down mag

        // operator+(Coord), operator-(Coord)
        c1 = Coord::new(0, 0) + Coord::new(6, 8);
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 6);
        assert_eq!(y, 8);
        c1 = Coord::new(-70, 20) + Coord::new(10, -10);
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -60);
        assert_eq!(y, 10);
        c1 = Coord::new(-70, 20) - Coord::new(10, -10);
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -80);
        assert_eq!(y, 30);

        // operator*(int)
        c1 = Coord::new(0, 0) * 1;
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c1 = Coord::new(1, 1) * -5;
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -5);
        assert_eq!(y, -5);
        c1 = Coord::new(11, 5) * -5;
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -55);
        assert_eq!(y, -25);

        // operator+(Dir), operator-(Dir)
        c1 = Coord::new(0, 0);
        let mut c2 = c1 + Dir::new(Compass::Center);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c2 = c1 + Dir::new(Compass::E);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        c2 = c1 + Dir::new(Compass::W);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, -1);
        assert_eq!(y, 0);
        c2 = c1 + Dir::new(Compass::SW);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, -1);
        assert_eq!(y, -1);

        c2 = c1 - Dir::new(Compass::Center);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c2 = c1 - Dir::new(Compass::E);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, -1);
        assert_eq!(y, 0);
        c2 = c1 - Dir::new(Compass::W);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        c2 = c1 - Dir::new(Compass::SW);
        let (x, y) = coord_fields(c2);
        assert_eq!(x, 1);
        assert_eq!(y, 1);

        // raySameness()
        c1 = Coord::new(0, 0);
        c2 = Coord::new(10, 11);
        let d1 = Dir::new(Compass::Center);
        assert!(are_close_f(c1.ray_sameness(c2), 1.0)); // special case - zero vector
        assert!(are_close_f(c2.ray_sameness(c1), 1.0)); // special case - zero vector
        assert!(are_close_f(c2.ray_sameness_dir(d1), 1.0)); // special case - zero vector
        c1 = c2;
        assert!(are_close_f(c1.ray_sameness(c2), 1.0));
        assert!(are_close_f(Coord::new(-10, -10).ray_sameness(Coord::new(10, 10)), -1.0));
        c1 = Coord::new(0, 11);
        c2 = Coord::new(20, 0);
        assert!(are_close_f(c1.ray_sameness(c2), 0.0));
        assert!(are_close_f(c2.ray_sameness(c1), 0.0));
        c1 = Coord::new(0, 444);
        c2 = Coord::new(113, 113);
        assert!(are_close_f(c1.ray_sameness(c2), 0.707106781));
        c2 = Coord::new(113, -113);
        assert!(are_close_f(c1.ray_sameness(c2), -0.707106781));
    }

    #[test]
    fn test_polar_basic_types() {
        // Polar ctor from mag, dir
        let mut p1 = Polar::new(0, Compass::S);
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 0);
        assert_eq!(dir, Compass::S);
        p1 = Polar::new(10, Compass::SE);
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, 10);
        assert_eq!(dir, Compass::SE);
        p1 = Polar::new(-10, Compass::NW);
        let (mag, dir) = polar_fields(p1);
        assert_eq!(mag, -10);
        assert_eq!(dir, Compass::NW);

        // .asCoord()
        let mut c1 = Polar::new(0, Compass::Center).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c1 = Polar::new(20, Compass::Center).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        c1 = Polar::new(20, Compass::N).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 0);
        assert_eq!(y, 20);
        p1 = Polar::new(12, Compass::W);
        c1 = p1.as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -12);
        assert_eq!(y, 0);
        c1 = Polar::new(14, Compass::NE).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 10);
        assert_eq!(y, 10);
        c1 = Polar::new(-14, Compass::NE).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -10);
        assert_eq!(y, -10);
        c1 = Polar::new(14, Compass::E).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, 14);
        assert_eq!(y, 0);
        c1 = Polar::new(-14, Compass::E).as_coord();
        let (x, y) = coord_fields(c1);
        assert_eq!(x, -14);
        assert_eq!(y, 0);
    }
}
