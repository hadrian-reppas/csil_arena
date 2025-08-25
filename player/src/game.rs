#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cell {
    Color(Color),
    Me,
    Opponent,
}

impl Cell {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0..=7 => Some(Self::Color(Color::from_u8(v).unwrap())),
            8 => Some(Self::Me),
            9 => Some(Self::Opponent),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::Color(c) => c.to_u8(),
            Self::Me => 8,
            Self::Opponent => 9,
        }
    }

    pub fn is_color(&self) -> bool {
        matches!(self, Self::Color(_))
    }

    pub fn unwrap_color(&self) -> Color {
        if let Self::Color(color) = self {
            *color
        } else {
            panic!("called `Cell::unwrap_color()` on {self:?}");
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum Color {
    Red = 0,
    Orange = 1,
    Yellow = 2,
    Green = 3,
    Cyan = 4,
    Blue = 5,
    Purple = 6,
    Pink = 7,
}

impl Color {
    pub const ALL: [Color; 8] = [
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Cyan,
        Self::Blue,
        Self::Purple,
        Self::Pink,
    ];

    pub fn from_u8(v: u8) -> Option<Self> {
        Self::ALL.get(v as usize).copied()
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}
