#![allow(dead_code)]

use rl2025::*;



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
    (5, [Grass,Grass,Grass,Grass,None]),
    (5, [Town,Town,Town,Town,None]),
    (5, [Forest,Forest,Forest,Forest,None]),
    (5, [River,River,Town,Town,None]),
    (5, [Road,Forest,Road,Grass,Road]),
    (5, [Forest,Grass,Grass,Town,None]),
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

    let mut sim = SimulationState::new();
    sim.next_tile();


    let mut resources = Resources::new(ASSETS);
    for path in LOAD_ME { resources.load_texture(path, FilterMode::Nearest); }

    let display_dim: Vec2 = DISPLAY_GRID.dim();
    let mut display = Display::new(resources, display_dim);

    loop {
      if let Some(input) = get_input() {
        //debug!("{:?}", input);
        // get input and advance state
        match input {
          Input::Dir(dir4) => {
            // move player
            sim.player_pos += dir4.into();

            // place tile
            if sim.board[sim.player_pos] == Tile::default() {
              sim.board[sim.player_pos] = sim.player_next_tile;
              sim.next_tile();
              debug!("tiles left: {:?}", sim.player_tiles);
            }
          },
          Input::Rotate1 => {
            // TODO
          }
          Input::Rotate2 => {
            //TODO
          }
          Input::Discard => {
            //TODO
          }
          Input::LevelUp => {
            //TODO
          }
        }

        //debug!("{:?}", sim.player_pos);
        let camera_offset: IVec = display.camera_focus - sim.player_pos;
        display.camera_focus = sim.player_pos + CAMERA_TETHER.clamp_pos(camera_offset);

      }
      let scale: f32 = f32::min(
        screen_width() / display.dim.x as f32,
        screen_height() / display.dim.y as f32,
      );

      { // Redraw the display
        set_camera(&display.render_to);
        clear_background(DARKGRAY);


        // DEBUG GRID VERTICES
        let spots = IRect{x: 0, y: 0, width: 11, height: 11};
        for s in spots.iter() {
          let v = Vec2::from(s) * 128.;
          draw_circle(v.x, v.y, 20., BLUE);
        }

        // Draw terrain
        for offset in (IRect{ x: -8, y:-8, width: 17, height: 17}).iter() {
          let p = sim.player_pos + offset;
          let tile = sim.board[p];
          for &terrain in Terrain::DRAW_ORDER {
            // is there a pair of adjacent sides of this terrain type?
            let mut adjacent = false;
            // is there a pair of opposite sides of this terrain type?
            let mut opposite = false;
            for d in Dir4::list() {
              if tile.contents[d.index()] != terrain {
                continue;
              }
              let n = d.rotate4(1);
              if tile.contents[n.index()] == terrain {
                adjacent = true;
              }
              let o = d.opposite();
              if tile.contents[o.index()] == terrain {
                opposite = true;
              }
            }

            if adjacent {
              // any adjacency implies triangle
              for d in Dir4::list() {
                if tile.contents[d.index()] != terrain { continue; }
                let img = terrain_triangle(terrain, d);
                display.draw_tile(p.into(), terrain.color(), &img);
              }
            } else if opposite && tile.contents[4] == terrain {
              // no adjacency + opposite + center implies bridge
              for d in Dir4::list() {
                if tile.contents[d.index()] != terrain { continue; }
                let img = terrain_bridge(terrain, d);
                display.draw_tile(p.into(), terrain.color(), &img);
                break; // a single bridge image covers both directions
              }
            } else {
              // fallthrough is wedge
              for d in Dir4::list() {
                if tile.contents[d.index()] != terrain { continue; }
                let img = terrain_wedge(terrain, d);
                display.draw_tile(p.into(), terrain.color(), &img);
              }

            }
            // TODO:
            // draw special center item if present
            // eg quest

          }
        }

        // Draw player
        display.draw_tile(
          sim.player_pos.into(),
          RED,
          HERO
        );
      }

      { // Copy the display to the screen
        set_default_camera();
        clear_background(BLACK);

        draw_texture_ex(
          &display.texture,
          (screen_width() - (scale * display.dim.x as f32)) * 0.5,
          (screen_height() - (scale * display.dim.y as f32)) * 0.5,
          WHITE,
          DrawTextureParams {
            dest_size: Some(vec2(
                           scale * display.dim.x as f32,
                           scale * display.dim.y as f32,
                       )),
                       flip_y: true,
                       ..Default::default()
          },
        );
      }




      next_frame().await
    }
}


