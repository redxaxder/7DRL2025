#![allow(dead_code)]

use rl2025::*;



mod tiles {
  use crate::*;
  use Terrain::*;

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
    let g: D8 = D8::list()[rng.next_u64() as usize % 8];
    g * Tile { contents : TABLE[i].1 }

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
  player_tile_transform: D8,

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
      player_tile_transform: D8::E,
      board: Buffer2D::new(Tile::default(), BOARD_RECT),
      enemies: Map::new(),
      rng: from_global_rng(),
    }
  }

  pub fn player_level_up(&mut self) {
    if self.player_xp < self.player_xp_next() { return; }
    self.player_xp -= self.player_xp_next();
    self.player_hp_max += 1;
    self.player_hp = self.player_hp_max;
    self.player_level += 1;
  }

  pub fn player_xp_next(&self) -> i64 {
    self.player_level * 3
  }


  pub fn player_current_tile(&self) -> Tile {
    self.player_tile_transform * self.player_next_tile
  }

  pub fn next_tile(&mut self) {
    self.player_next_tile = tiles::generate(&mut self.rng);
    self.player_tile_transform = D8::E;
    self.player_tiles -= 1;
  }

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

            // try to place tile
            if sim.board[sim.player_pos] == Tile::default() {
              sim.board[sim.player_pos] = sim.player_current_tile();
              sim.next_tile();
              //debug!("tiles left: {:?}", sim.player_tiles);
            }
          },
          Input::Rotate1 => {
            sim.player_tile_transform = D8::R1 * sim.player_tile_transform;
          }
          Input::Rotate2 => {
            sim.player_tile_transform = D8::R3 * sim.player_tile_transform;
          }
          Input::Discard => {
            sim.next_tile();
            //debug!("tiles left: {:?}", sim.player_tiles);
          }
          Input::LevelUp => {
            sim.player_level_up()
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
        clear_background(DARKPURPLE);


        // DEBUG GRID VERTICES
        let spots = IRect{x: 0, y: 0, width: 20, height: 20};
        for s in spots.iter() {
          let v = Vec2::from(s) * 128.;
          draw_circle(v.x, v.y, 20., BLUE);
        }

        // Draw terrain
        for offset in (IRect{ x: -8, y:-8, width: 17, height: 17}).iter() {
          let p = sim.player_pos + offset;
          let tile = sim.board[p];
          let r = DISPLAY_GRID.rect(p - display.camera_focus);
          display.draw_tile(r, tile);
        }

        // Draw player
        display.draw_grid(
          sim.player_pos.into(),
          RED,
          HERO
        );

        { // Draw HUD
          let font_size = 100;
          let font_scale = 1.;

          let margin = 15.;
          let sz = DISPLAY_GRID.tile_size;
          let hudbar_height = sz.y + 2. * margin;
          let hud_top = display.dim.y - hudbar_height;
          draw_rectangle(0.,hud_top,display.dim.x, hudbar_height, DARKGRAY);


          // Next Tile
          let r = Rect {
            x: display.dim.x - sz.x - margin,
            y: hud_top + margin ,
            w: sz.x,
            h: sz.y
          };
          display.draw_tile(r, sim.player_current_tile());

          // Remaining tiles
          let remaining_tiles = format!("{}", sim.player_tiles);
          let textdim: TextDimensions = measure_text(&remaining_tiles, None, font_size, font_scale);
          let leftover = hudbar_height - textdim.height;
          let x = r.x - textdim.width - margin;
          let y = hud_top + (0.5 * leftover) + textdim.offset_y;
          draw_text(&remaining_tiles, x, y, font_size as f32, WHITE);


          let mut cursor = margin;
          // Current/Max HP
          let hp = format!("HP: {}/{} ", sim.player_hp, sim.player_hp_max);
          let textdim: TextDimensions = measure_text(&hp, None, font_size, font_scale);
          let leftover = hudbar_height - textdim.height;
          let y = hud_top + (0.5 * leftover) + textdim.offset_y;
          draw_text(&hp, cursor, y, font_size as f32, WHITE);
          cursor += textdim.width + margin;


          // Current/Next XP
          let xp = format!("XP: {}/{}", sim.player_xp, sim.player_xp_next());
          let textdim: TextDimensions = measure_text(&xp, None, font_size, font_scale);
          let leftover = hudbar_height - textdim.height;
          let y = hud_top + (0.5 * leftover) + textdim.offset_y;
          draw_text(&xp, cursor, y, font_size as f32, WHITE);
          //cursor += textdim.width + margin;


        }


      }

      { // Copy the display to the screen
        set_default_camera();
        clear_background(BLACK);

        draw_texture_ex(
          &display.texture,
          (screen_width() - (scale * display.dim.x)) * 0.5,
          (screen_height() - (scale * display.dim.y)) * 0.5,
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


