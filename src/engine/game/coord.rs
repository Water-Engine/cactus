use std::ops::{Add, Mul, Sub};

use crate::engine::game::board::{FILE_NAMES, RANK_NAMES};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Coord {
    pub file_idx: i32,
    pub rank_idx: i32,
}

impl From<(i32, i32)> for Coord {
    fn from((file_idx, rank_idx): (i32, i32)) -> Self {
        Self {
            file_idx: file_idx,
            rank_idx: rank_idx,
        }
    }
}

impl From<i32> for Coord {
    fn from(square_idx: i32) -> Self {
        Self::new(square_idx)
    }
}

impl From<String> for Coord {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl Into<i32> for Coord {
    fn into(self) -> i32 {
        self.index()
    }
}

impl Into<String> for Coord {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Coord {
    pub fn new(square_idx: i32) -> Self {
        Self {
            file_idx: Self::file_of_square(square_idx),
            rank_idx: Self::rank_of_square(square_idx),
        }
    }

    pub const fn from((file_idx, rank_idx): (i32, i32)) -> Self {
        Self {
            file_idx: file_idx,
            rank_idx: rank_idx,
        }
    }

    pub fn is_light_square(&self) -> bool {
        (self.file_idx + self.rank_idx) % 2 != 0
    }

    pub fn is_valid_square(&self) -> bool {
        self.file_idx >= 0 && self.file_idx < 8 && self.rank_idx >= 0 && self.rank_idx < 8
    }

    pub fn index(&self) -> i32 {
        self.rank_idx * 8 + self.file_idx
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        s.push(FILE_NAMES[self.file_idx as usize]);
        s.push_str(&(self.rank_idx + 1).to_string());
        s
    }

    pub fn from_string(name: String) -> Self {
        Self {
            file_idx: FILE_NAMES
                .iter()
                .position(|&file| name.contains(file))
                .map(|i| i as i32)
                .unwrap_or(-1),
            rank_idx: RANK_NAMES
                .iter()
                .position(|&rank| name.contains(rank))
                .map(|i| i as i32)
                .unwrap_or(-1),
        }
    }

    pub fn rank_of_square(square_idx: i32) -> i32 {
        square_idx >> 3
    }

    pub fn file_of_square(square_idx: i32) -> i32 {
        square_idx & 0b000111
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Coord::from((self.file_idx + rhs.file_idx, self.rank_idx + rhs.rank_idx))
    }
}

impl Add for &Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Coord::from((self.file_idx + rhs.file_idx, self.rank_idx + rhs.rank_idx))
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord::from((self.file_idx - rhs.file_idx, self.rank_idx - rhs.rank_idx))
    }
}

impl Sub for &Coord {
    type Output = Coord;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord::from((self.file_idx - rhs.file_idx, self.rank_idx - rhs.rank_idx))
    }
}

impl Mul<i32> for Coord {
    type Output = Coord;

    fn mul(self, rhs: i32) -> Self::Output {
        Coord::from((self.file_idx * rhs, self.rank_idx * rhs))
    }
}

impl Mul<i32> for &Coord {
    type Output = Coord;

    fn mul(self, rhs: i32) -> Self::Output {
        Coord::from((self.file_idx * rhs, self.rank_idx * rhs))
    }
}
