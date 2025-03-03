#![allow(dead_code)]

use rl2025::*;

#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
enum Terrain {
  None,
  Grass,
  //Cave,
  //Town,
  //River,
  //Road,
  //Forest,
  //Quest,
}



#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
struct Tile {
  // right up left down (matching dir4.index)
  // followed by center in index 4
  contents: [Terrain; 5]
}

mod tiles {
  use crate::Terrain;
  use crate::Tile;
  use crate::Rng;
  use crate::Terrain::*;
  const TABLE: &[(usize, [Terrain;5])] = &[
    (5, [Grass,Grass,Grass,Grass,None])
    ];

  const fn weight() -> usize {
    let mut r = 0;
    let n = TABLE.len();
    let mut i = 0;
    while i < n {
      let (w,_) = TABLE[i];
      r += w;
      i += 1;
    }
    r
  }

  const TABLE_WEIGHT: usize = weight();

  pub fn generate(rng: &mut Rng) -> Tile {
    let mut w = rng.next_u64() as usize % TABLE_WEIGHT;
    let mut i = 0;
    while w > TABLE[i].0 {
      w -= TABLE[i].0;
      i += 1;
    }
    Tile { contents : TABLE[i].1 }

  }

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
    todo!()
  }
}

#[derive(Clone)]
struct SimulationState {
  player_pos: Position,
  player_hp: i64,
  player_hp_max: i64,
  player_xp: i64,
  player_level: i64,
  player_tiles: i64,
  player_next_tile: Tile,

  board: Buffer2D<Tile>,
  enemies: Map<Position, Enemy>,

  rng: Rng,
}

const BOARD_RECT: IRect = IRect { x: 0, y:0, width: 50, height: 50 };

impl SimulationState {
  pub fn new() -> Self {
    SimulationState {
      player_pos: IVec::ZERO,
      player_hp: 5,
      player_hp_max: 5,
      player_xp: 0,
      player_level: 1,
      player_tiles: 30,
      player_next_tile: Tile::default(),
      board: Buffer2D::new(Tile::default(), BOARD_RECT),
      enemies: Map::new(),
      rng: from_global_rng(),
    }
  }

  pub fn next_tile(&mut self) {
    self.player_next_tile = tiles::generate(&mut self.rng);
    self.player_tiles -= 1;
  }

}

type EnemyId = u64;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
enum EnemyType {
  One,
  Two,
  Tree,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
struct Enemy {
  id: EnemyId,
  pos: Position,
  t: EnemyType
}


#[macroquad::main("7drl")]
async fn main() {
    debug!("This is a debug message");
    info!("and info message");
    error!("and errors, the red ones!");
    warn!("Or warnings, the yellow ones.");

    let mut game = SimulationState::new();
    game.next_tile();



    loop {
        clear_background(LIGHTGRAY);

        debug!("Still alive!");

        next_frame().await
    }
}


