pub mod display;
pub use display::*;

pub mod animation;
pub use crate::animation::*;

pub mod random;
pub use crate::random::*;

pub mod geometry;
pub use crate::geometry::*;

pub mod input;
pub use crate::input::*;


pub mod resources;
pub use resources::*;

pub mod util;
pub use util::*;

pub mod fov;


pub use macroquad::prelude::*;

pub type Seconds = f64;

pub mod footguns;


pub mod misc;

pub mod assets;
pub use crate::assets::*;


#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub enum Terrain {
  None,
  Grass,
  Town,
  River,
  Road,
  Forest,
  //Cave,
  //Quest,
}
impl Terrain {
  pub const DRAW_ORDER: &[Self] = &[
  Self::Grass,
  Self::Town,
  Self::River,
  Self::Road,
  Self::Forest,
  //Self::Cave,
  //Self::Quest,
  ];

  pub fn index(self) -> usize {
    unsafe {
      std::mem::transmute::<Self, u8>(self) as usize
    }
  }

  pub fn color(self) -> Color {
    TERRAIN_COLOR[self.index()]
  }
}


const TERRAIN_COLOR: &[Color] = &[
  BLACK,
  GREEN,
  ORANGE,
  BLUE,
  WHITE,
  DARKGREEN,
  RED,
  YELLOW,
];
