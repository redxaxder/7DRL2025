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

use linear_map::*;


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


#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Tile {
  // right up left down (matching dir4.index)
  // followed by center in index 4
  pub contents: [Terrain; 5]
}

impl Default for Tile {
  fn default() -> Self {
    Tile {
      contents: [Terrain::None;5]
    }
  }
}

impl std::ops::Mul<Tile> for D8 {
  type Output = Tile;
  fn mul(self, rhs: Tile) -> Self::Output {
    let mut contents = rhs.contents.clone();
    for d in Dir4::list() {
      let d2 = self * d;
      contents[d2.index()] = rhs.contents[d.index()];
    }
    Tile {contents}
  }
}

pub type EnemyId = u64;

#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub enum EnemyType {
  Clyde = 0, //moves randomly
  Blinky = 1, //chases player
  Pinky = 2, //avoids other enemies
  GhostWitch = 3, //the boss
}

impl EnemyType {
  pub fn list() -> [Self;3] {
    unsafe {
      core::array::from_fn(|x| core::mem::transmute(x as u8))
    }
  }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Enemy {
  pub id: EnemyId,
  pub t: EnemyType
}

impl Enemy {
  pub fn new(rng: &mut Rng, nme_type: EnemyType) -> Self {
    let id = rng.next_u64();
    let t = nme_type;
    Enemy { id, t}
  }
}

pub type DMap = Buffer2D<i16>;

#[derive(Clone, Debug)]
pub struct WrapMap<V> {
  rect: IRect,
  map: Map<Position, V>
}
impl<V> WrapMap<V> {
  pub fn new(rect: IRect) -> Self {
    WrapMap { rect, map: Map::new() }
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn insert(&mut self, k: IVec, v: V) {
    self.map.insert(
      self.rect.wrap(k),
      v
    );
  }

  pub fn get(&mut self, k: IVec) -> Option<&V> {
    self.map.get(
      &self.rect.wrap(k)
    )
  }

  pub fn remove(&mut self, k: IVec) -> Option<V> {
    self.map.remove(
      &self.rect.wrap(k)
    )
  }
  pub fn contains_key(&mut self, k: IVec) -> bool {
    self.map.contains_key(
      &self.rect.wrap(k)
    )
  }
  pub fn keys(&mut self) -> Keys<IVec, V> {
    self.map.keys()
  }

  pub fn iter(&self) -> Iter<IVec, V> {
    self.map.iter()
  }

}

impl<V> std::ops::Index<IVec> for WrapMap<V> {
    type Output = V;

    fn index(&self, index: IVec) -> &Self::Output {
      &self.map[&self.rect.wrap(index)]
    }
}
