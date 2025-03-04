#![allow(dead_code)]

use rl2025::*;

type RegionId = u16;
const BOARD_RECT: IRect = IRect { x: 0, y:0, width: 50, height: 50 };
const MONSTER_SPAWN_CHANCE: u64 = 20; // units are 1/10 percent
const QUEST_SPAWN_CHANCE: u64 = 5; // units are 1/10 percent
const MIN_QUEST: u64 = 5;
const MAX_QUEST: u64 = 20;
const REGION_REWARD_THRESHOLD: usize = 4;

#[derive(Clone)]
struct SimulationState {
  player_pos: Position,
  player_hp: i64,
  player_hp_max: i64,
  player_xp: i64,
  player_level: i64,
  player_tiles: i64,
  player_next_tile: Tile,
  next_tile_has_quest: bool,
  player_tile_transform: D8,
  player_speed_penalty: i64,

  board: Buffer2D<Tile>,
  regions: Buffer2D<[RegionId;4]>,
  region_sizes: Map<RegionId, usize>,
  // the subposition of the first tile in this region
  region_start: Map<RegionId, Subposition>,
  next_region_id: RegionId,
  // regions that border void
  open_regions: Set<RegionId>,
  // positions bordering void
  void_frontier: Set<Position>,

  enemies: WrapMap<Enemy>,
  rng: Rng,

  quests: WrapMap<Quest>,

  player_dmap: DMap,
  nearest_enemy_dmap: DMap,
}
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
      next_tile_has_quest: false,
      player_tile_transform: D8::E,
      player_speed_penalty: 0,
      board: Buffer2D::new(Tile::default(), BOARD_RECT),
      enemies: WrapMap::new(BOARD_RECT),
      quests: WrapMap::new(BOARD_RECT),
      regions: Buffer2D::new([RegionId::MAX;4], BOARD_RECT),
      next_region_id: 1,
      open_regions: Set::new(),
      void_frontier: Set::new(),
      region_sizes: Map::new(),
      region_start: Map::new(),
      rng: from_current_time(),
      player_dmap: Buffer2D::new(0, BOARD_RECT),
      nearest_enemy_dmap: Buffer2D::new(0, BOARD_RECT),
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

  fn fill_region_ids(&mut self, position: Position, dir: Dir4) {
    let mut frontier: Vec<(Position, Dir4)> = vec!( (position, dir));
    while let Some((p,d)) = frontier.pop() {
      let rid = self.regions[p][d.index()];
      let t0 = self.board[p].contents[d.index()];

      let neighbors = [
        // Example with d = Right
        // |-----------------|-----------------|
        // | \             / | \             / |
        // |   \    n1   /   |   \         /   |
        // |     \     /     |     \     /     |
        // |       \ /       |       \ /       |
        // | n0    / \  (p,d)| n3    / \       |
        // |     /     \     |     /     \     |
        // |   /    n2   \   |   /         \   |
        // | /             \ | /             \ |
        // |-----------------|-----------------|
        // note: n0 is special cased to only be a neighbor if center terrain matches
        (p, d.opposite()),
        (p, d.rotate4(1)),
        (p, d.rotate4(3)),
        (p + d.into(), d.opposite())
      ];

      let mut min_rid = RegionId::MAX;

      for i in 0..4 { // find the greatest region id among matching neighbors
        let (np, nd) = neighbors[i];
        // the opposite is not considered adajcent if the center terrain doesn't match
        if i == 0 && self.board[p].contents[4] != t0 { continue; }
        let t1 = self.board[np].contents[nd.index()];
        if t1 != t0 { continue; }
        let rid1 = self.regions[np][nd.index()];

        min_rid = min_rid.min(rid1);
      }

      for i in 0..4 { // walk matches with rid above min
        let (np, nd) = neighbors[i];
        // the opposite is not considered adajcent if the center terrain doesn't match
        if i == 0 && self.board[p].contents[4] != t0 { continue; }
        let t1 = self.board[np].contents[nd.index()];
        if t1 != t0 { continue; }
        let rid1 = self.regions[np][nd.index()];

        if min_rid < rid1 { frontier.push((np, nd)) }
      }

      if min_rid < rid {
        // if two compatible regions with distinct rids are adjacent,
        // the one with the larger id is merged into the smaller
        self.regions[p][d.index()] = min_rid;
        // we remove the larger from the rid start tracker
        if self.region_start.remove(&rid).is_some() {
          debug!("merged region {}", rid);
        }

        //if rid < RegionId::MAX {
        //  debug!("update cell regionid {} -> {}", rid, min_rid);
        //}
      }
    }
  }

  pub fn place_tile(&mut self, position: Position, tile: Tile) {
    self.board[position] = tile;

    { // region tracking
      // merge regions
      for d in Dir4::list() {
        self.fill_region_ids(position, d);
      }

      // new regions
      for d in Dir4::list() {
        self.fill_region_ids(position,d);
        if self.regions[position][d.index()] == RegionId::MAX {
          self.regions[position][d.index()] = self.next_region_id;
          self.region_start.insert(self.next_region_id, (position, d));
          self.next_region_id += 1;
        }
      }

      // update void frontier
      let wp = self.board.rect.wrap(position);
      self.void_frontier.remove(&wp);
      for d in Dir4::list() {
        let n = self.board.rect.wrap(wp + d.into());
        if self.board[n] == Tile::default() {
          self.void_frontier.insert(n);
        }
      }

      // rebuild open regions
      self.open_regions.clear();
      for &void_cell in &self.void_frontier {
        for d in Dir4::list() {
          let cell = void_cell + d.into();
          let regionid = self.regions[cell][d.opposite().index()];
          if regionid < RegionId::MAX {
            self.open_regions.insert(regionid);
          }
        }
      }
    }

    { // check for perfect tile bonuses
      // on placed tile and neighbors
      let mut to_check = vec!(position);
      for d in Dir4::list() {
        to_check.push(position + d.into());
      }
      for &p in &to_check {
        let mut is_matched = true;
        let ptile = self.board[p];

        for d in Dir4::list() {
          let ntile = self.board[p + d.into()];
          if ptile.contents[d.index()]
            != ntile.contents[d.opposite().index()] {
            is_matched = false;
          }
        }
        if is_matched {
          // perfect tile bonus
          // TODO: UI hint
          self.player_tiles += 1;
          debug!("perfect tile bonus");
        }
      }
    }
  }

  pub fn update_region_sizes(&mut self) {
    self.region_sizes.clear();
    let mut v = vec![];
    for p in BOARD_RECT.iter() {
      v.clear();
      for d in Dir4::list() {
        let rid = self.regions[p][d.index()];
        // let terrain = self.board[p].contents[d.index()];
        if rid == RegionId::MAX { continue; }
        v.push(rid);
      }
      v.sort();
      v.dedup();
      // FIXME: algorithm is quadratic in region count
      // maybe replace linear map with hashmap
      for &rid in &v {
        *self.region_sizes.entry(rid).or_insert(0) += 1;
      }
    }
  }

  pub fn reward_completed_region(&mut self, rid: RegionId) {
    //TODO: actual reward

    let (position, dir) = self.region_start[&rid];
    let terrain = self.board[position].contents[dir.index()];
    let size = self.region_sizes[&rid];
    if terrain == Terrain::River {
      // TODO: special case river rewards
    } else {
      let xp_reward = size.saturating_sub(REGION_REWARD_THRESHOLD);
      if xp_reward > 0 {
        // TODO: UI hints
        self.player_xp += xp_reward as i64;
        self.player_tiles += 1;
        debug!("region reward: {} xp 1 tile", xp_reward);
      }

    }
  }

  pub fn player_current_tile(&self) -> Tile {
    self.player_tile_transform * self.player_next_tile
  }

  pub fn next_tile(&mut self) {
    self.player_next_tile = tiles::generate(&mut self.rng);
    self.player_tile_transform = D8::E;
    self.player_tiles -= 1;

    // does the next tile have a quest?
    if roll_chance(&mut self.rng, QUEST_SPAWN_CHANCE) {
      if let Some(quest) = eligible_for_quest(&self.enemies, &self.quests, &mut self.rng) {
      }
    }
  }

  pub fn update_player_dmap(&mut self) {
    self.player_dmap.fill(i16::MAX);
    let mut d = 0;
    let mut frontier = Vec::new();
    frontier.push(self.player_pos);
    let mut next_frontier = Vec::new();

    loop {
      while let Some(visit) = frontier.pop() {
        if self.player_dmap[visit] > d {
          self.player_dmap[visit] = d;
          for d in Dir4::list() {
            let neighbor = visit + d.into();
            if self.player_dmap[neighbor] == i16::MAX 
              && self.board[neighbor] != Tile::default()
            {
              next_frontier.push(neighbor);
            }
          }
        }
      }
      if next_frontier.len() == 0 {
        break;
      }
      next_frontier.sort();
      next_frontier.dedup();
      std::mem::swap(&mut frontier, &mut next_frontier);
      d += 1;
    }
  }

  pub fn update_nearest_dmap(&mut self) {
    self.nearest_enemy_dmap.fill(i16::MAX);
    let mut d = 0;
    let mut frontier = Vec::new();
    for (pos, nme) in self.enemies.iter() {
      if nme.t == EnemyType::Pinky {
        // pinkies shouldn't hide from each other
        continue;
      }
      frontier.push(*pos);
    }
    let mut next_frontier = Vec::new();

    loop {
      while let Some(visit) = frontier.pop() {
        if self.nearest_enemy_dmap[visit] > d {
          self.nearest_enemy_dmap[visit] = d;
          for d in Dir4::list() {
            let neighbor = visit + d.into();
            if self.nearest_enemy_dmap[neighbor] == i16::MAX
              && self.board[neighbor] != Tile::default()
            {
              next_frontier.push(neighbor);
            }
          }
        }
      }
      if next_frontier.len() == 0 { break; }
      next_frontier.sort();
      next_frontier.dedup();
      std::mem::swap(&mut frontier, &mut next_frontier);
      d += 1;
    }
  }

  pub fn move_enemy(&mut self, from: Position, to: Position) {
    if self.enemies.contains_key(from) && !self.enemies.contains_key(to) {
      self.enemies.insert(to, self.enemies[from]);
      self.enemies.remove(from);
    }
  }

  // 2- perfect match
  // 1- imperfect match
  // 0- missing required match
  pub fn tile_compatibility(&self, pos: Position, tile: Tile) -> u8 {
    let mut compat = 2;
    for d in Dir4::list() {
      let t1: Terrain = tile.contents[d.index()];
      let p2 = pos + d.into();
      let i2 = d.opposite().index();
      let t2: Terrain = self.board[p2].contents[i2];
      if t2 == Terrain::None { continue; } // fully compatible
      if t1 == t2 { continue; } // fully compatible
      if t1 != t2 {
        compat = compat.min(1); // soft mismatch
        if t1.requires_match() || t2.requires_match() {
          compat = 0; // hard mismatch
        }
      }
    }
    compat
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
  for path in LOAD_ME { resources.load_texture(path, FilterMode::Linear); }

  let display_dim: Vec2 = DISPLAY_GRID.dim();
  let mut display = Display::new(resources, display_dim);

  loop {
    let mut inputdir: Option<Dir4> = None;
    if let Some(input) = get_input() {
      match input {
        Input::Dir(dir) => {
          inputdir = Some(dir)
        }
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
    }

    let mut tile_placed: bool = false;
    let mut player_moved: bool = false;
    let mut in_combat = false;
    let mut needs_road = false;
    let mut can_move = true;
    if let Some(playermove) = inputdir  {
      let target = sim.player_pos + playermove.into();
      let target_empty = sim.board[target] == Tile::default();
      for d in Dir4::list() { // are we in combat?
        let adj = sim.player_pos + d.into();
        // monsters in void don't count
        if sim.board[adj] == Tile::default() { continue; }
        in_combat = in_combat || sim.enemies.get(adj).is_some();
      }

      // do combat

      if in_combat {
        let crowd: Map<Position, u8> = {
          // calculates the crowd of enemies we're trying to fight
          // each occupied position is put into the map, along with
          // how many steps away it is from the fight location
          let mut result = Map::new();
          let mut frontier: Vec<Position> = Vec::new();
          let mut next_frontier: Vec<Position> = Vec::new();
          let mut distance: u8 = 0;
          frontier.push(target);
          while frontier.len() > 0 {
            while let Some(cursor) = frontier.pop() {
              if result.contains_key(&cursor) { continue; }
              if !sim.enemies.contains_key(cursor) { continue; }
              if sim.board[cursor] == Tile::default() { continue; }
              result.insert(cursor, distance);
              for d in Dir4::list() {
                let neighbor = cursor + d.into();
                next_frontier.push(neighbor);
              }
            }
            std::mem::swap(&mut frontier, &mut next_frontier);
            distance += 1;
          }
          result
        };
        if crowd.len() > 0 { // fight!
          while let Some(_defeated) = sim.enemies.remove(target) {
            // enemy is defeated
            // player takes a hit
            sim.player_hp -= 1;
            sim.player_xp += 1;
            // enemies behind move up
            let mut vacated = target;
            let mut dist = 0;
            'scooch: loop {
              for d in Dir4::list() {
                let neighbor = vacated + d.into();
                if let Some(&dist2) = crowd.get(&neighbor) {
                  // enemies only want to scooch closer
                  if dist2 <= dist { continue; }
                  if let Some(new_challenger) = sim.enemies.remove(neighbor) {
                    sim.enemies.insert(vacated, new_challenger);
                    vacated = neighbor;
                    dist = dist2;
                    continue 'scooch;
                  }
                }
              }
              break;
            }
            player_moved = true;
          }
        } else { // nobody in this spot to fight
          needs_road = true;
        }
      }

      let using_road = {
        // two cases:
        // 1) there is an existing road here we can take
        // 2) there is a half road here, with the other half
        //    in hand and oriented the right way
        // either way, the check for the first half of the road is the same

        let d1 = playermove;
        let d2 = playermove.opposite();
        let first_half = Terrain::Road ==
          sim.board[sim.player_pos].contents[d1.index()];

        let second_half = Terrain::Road ==
          if target_empty {
            sim.player_current_tile().contents[d2.index()]
          } else {
            sim.board[target].contents[d2.index()]
          };

        first_half && second_half
      };

      can_move = can_move && (!needs_road || using_road);
      can_move = can_move && (!target_empty || sim.tile_compatibility(target, sim.player_current_tile()) > 0);
      if !player_moved && can_move { // move player

        let target_is_slow: bool = {
          let mut rivers = 0;
          let t = sim.board[target];
          for i in 0..4 {
            if t.contents[i] == Terrain::River { rivers += 1; }
          }
          rivers >= 2
        };
        let edge_is_slow: bool = {
          let t0 = sim.board[sim.player_pos]
            .contents[playermove.index()];
          let t1 = sim.board[target]
            .contents[playermove.opposite().index()];
          t0 == Terrain::River
            && t1 == Terrain::River
        };

        sim.player_pos = target;
        player_moved = true;
        //debug!("player: {:?}", sim.player_pos);

        // try to place tile
        if sim.board[sim.player_pos] == Tile::default() {
          sim.place_tile(sim.player_pos, sim.player_current_tile());
          sim.next_tile();
          tile_placed = true;
          if sim.enemies.contains_key(sim.player_pos) {
            // new tiles smoosh monsters
            sim.enemies.remove(sim.player_pos);
          }
          //debug!("tiles left: {:?}", sim.player_tiles);
          sim.update_region_sizes();

          // check for completed regions
          // a region was just completed if
          // 1) its id appears on either this tile or its subposition neighbors
          // 2) its id *does not* appear in open regions
          let mut just_completed = Set::new();
          for d in Dir4::list() {
            let p2 = sim.player_pos + d.into();
            let d2 = d.opposite();
            for regionid in [
              sim.regions[sim.player_pos][d.index()],
              sim.regions[p2][d2.index()]
            ] {
              if regionid == RegionId::MAX { continue; }
              if !sim.open_regions.contains(&regionid) {
                just_completed.insert(regionid);
              }
            }
          }
          for &regionid in just_completed.iter() {
            sim.reward_completed_region(regionid);
          }
        } else { // we stepped on an existing tile
          if (target_is_slow || edge_is_slow) && !using_road {
            sim.player_speed_penalty += 1;
          }
        }
      }
    }

    //debug!("{:?}", sim.player_pos);
    let camera_offset: IVec = display.camera_focus - sim.player_pos;
    display.camera_focus = sim.player_pos + CAMERA_TETHER.clamp_pos(camera_offset);

    {//monsters
      let mut monster_turns = 0;
      if tile_placed || (player_moved && sim.player_tiles < 1) {
        monster_turns = 1;
        monster_turns += sim.player_speed_penalty;
        sim.player_speed_penalty = 0;
        sim.update_player_dmap();
      }
      while monster_turns > 0 {
        sim.update_nearest_dmap();
        //do monster turn
        for (pos, _) in sim.enemies.clone().iter() {
          let maybe_pos = enemy_pathfind(&mut sim, *pos);
          if let Some(new_pos) = maybe_pos {
            sim.move_enemy(*pos, new_pos);
          }
          //debug!("a monster turn happened at {:?}", pos)
        }
        //spawn monsters maybe
        for &p in sim.void_frontier.iter() {
          if sim.enemies.contains_key(p) {
            // don't spawn a monster if there's already a monster
            continue;
          }
          if roll_chance(&mut sim.rng, MONSTER_SPAWN_CHANCE) {
            //spawn a monster in this tile
            let random_enemy_type =
              EnemyType::list()[(sim.rng.next_u32() % 3) as usize];
            let nme = Enemy::new(&mut sim.rng, random_enemy_type);
            sim.enemies.insert(p, nme);
            //debug!("spawned a monster {:?} at {:?}", nme.t, p)
          }
        }
        monster_turns -= 1;
      }
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

      // Draw enemies
      for offset in (IRect{ x: -8, y:-8, width: 17, height: 17}).iter() {
        let p = sim.player_pos + offset;
        let Some(nme) = sim.enemies.get(p) else {
          continue;
        };
        display.draw_grid(
          Vec2::from(p),
          BLACK,
          &enemy(nme.t)
        );
      }

      { // Draw HUD
        let font_size = 60;
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

        // Current/Max HP
        // Current/Next XP
        let hp = format!("HP: {}/{} ", sim.player_hp, sim.player_hp_max);
        let hpdim: TextDimensions = measure_text(&hp, None, font_size, font_scale);
        let xp = format!("XP: {}/{}", sim.player_xp, sim.player_xp_next());
        let xpdim: TextDimensions = measure_text(&xp, None, font_size, font_scale);
        let leftover = hudbar_height - hpdim.height - xpdim.height;
        let hpy = hud_top + (0.33 * leftover) + hpdim.offset_y;
        let xpy = hud_top + hpdim.height
          + (0.66 * leftover) + xpdim.offset_y;
        draw_text(&hp, margin, hpy, font_size as f32, WHITE);
        draw_text(&xp, margin, xpy, font_size as f32, WHITE);


        // Speed penalty
        if sim.player_speed_penalty > 0 {
          let penalty = format!("X {}", sim.player_speed_penalty +1);
          let pdim = measure_text(&penalty, None, font_size, font_scale);
          let y = hud_top + 0.5 * (hudbar_height - pdim.height) + pdim.offset_y;
          let x = 0.5 * (display.dim.x - pdim.width);
          draw_text(&penalty,x,y, font_size.into(), WHITE);
        }

      }

      { // draw dmap2
        // let dmap = &sim.nearest_enemy_dmap;
        // for offset in (IRect{ x: -8, y:-8, width: 17, height: 17}).iter() {
        //  let p = sim.player_pos + offset;
        //  let dmapvalue = dmap[p];
        //  if dmapvalue > 20 {
        //    continue;
        //  }
        //  //let tile = sim.board[p];
        //  let r = DISPLAY_GRID.rect(p - display.camera_focus);
        //  let number = format!("{}", dmapvalue);
        //  let font_size = 50;
        //  let textdim: TextDimensions = measure_text(
        //    &number,
        //    None,
        //    font_size,
        //    1.
        //  );
        //  let leftoverx = r.w - textdim.width;
        //  let leftovery = r.h - textdim.height;
        //  let px = r.x + (0.5 * leftoverx);
        //  let py = r.y + (0.5 * leftovery) + textdim.offset_y + 30.;
        //  draw_text(&number, px, py, font_size as f32, WHITE);
        // }
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

fn select_candidate(mut candidates: Vec<Position>, sim: &mut SimulationState) -> Option<Position> {
  // filter out invalid tiles
  let mut valid: Vec<IVec> = Vec::new();
  for c in candidates.drain(0..) {
    if sim.board[c] != Tile::default() && !sim.enemies.contains_key(c) {
      valid.push(c);
    }
  }
  if valid.len() > 0 {
    Some(valid[sim.rng.next_u32() as usize % valid.len()])
  }
  else {
    None
  }
}

fn enemy_pathfind(sim: &mut SimulationState, pos: Position) -> Option<Position> {
  // handle the just-spawned void-camping case
  
  if sim.board[pos] == Tile::default() {
    let mut candidates: Vec<IVec> = Vec::new();
    for d in Dir4::list() {
      let candidate = pos + IVec::from(d);
      if sim.board[candidate] != Tile::default() {
        candidates.push(candidate);
      }
    }
    return select_candidate(candidates, sim);
  }

  // add forest edges to valid set

  let mut valid_1: Vec<Dir4> = forest_edges(&pos, &sim.board); 
  if valid_1.len() == 0 {
    // no forest edges means anything is a candidate
    valid_1 = Dir4::list().into();
  }

  // remove town edges from the valid set

  let town: Vec<Dir4> = town_edges(&pos, &sim.board);
  let valid: Vec<Dir4> = valid_1.iter()
    .filter(|e| !town.contains(e))
    .map(|x| *x)
    .collect();

  // pathfind given the valid set

  let mut candidates: Vec<IVec> = Vec::new();
  match sim.enemies[pos].t {
    EnemyType::Clyde => {
      for d in valid {
        let candidate = pos + IVec::from(d);
        if sim.board[candidate] != Tile::default() {
          candidates.push(candidate);
        }
      }
    }
    EnemyType::Blinky => {
      let mut min_dir: Dir4 = Dir4::Right;
      let mut min: i16 = i16::max_value();
      for d in valid {
        let c = pos + IVec::from(d);
        if sim.player_dmap[c] < min && sim.board[c] != Tile::default() {
          min = sim.player_dmap[c];
          min_dir = d;
        }
      }
      candidates.push(pos + min_dir.into());
    }
    EnemyType::Pinky => {
      let mut max_dir: Dir4 = Dir4::Right;
      let mut max: i16 = 0;
      for d in valid {
        let c = pos + IVec::from(d);
        if sim.nearest_enemy_dmap[c] > max && sim.board[c] != Tile::default() {
          max = sim.nearest_enemy_dmap[c];
          max_dir = d;
        }
      }
      candidates.push(pos + max_dir.into());
      // debug!("Pinky candidate {:?}", pos + max_dir.into());
    }
    EnemyType::GhostWitch => {
      // boss does not move
      candidates.push(pos);
    }
  }
  select_candidate(candidates, sim)
}

pub fn forest_edges(pos: &Position, board: &Buffer2D<Tile>) -> Vec<Dir4> {
  // right up left down (matching dir4.index)
  let mut candidates: Vec<Dir4> = Vec::new();
  let tile: Tile = board[*pos];
  for ix in 0..4 {
    let dir: Dir4 = Dir4::list()[ix];
    let neighbor: Tile = board[*pos + dir.into()];
    let edge1 = tile.contents[ix];
    let edge2 = neighbor.contents[dir.opposite().index()];
    if edge1 == Terrain::Forest && edge2 == Terrain::Forest {
      candidates.push(dir);
    }
  }
  candidates
}

pub fn town_edges(pos: &Position, board: &Buffer2D<Tile>) -> Vec<Dir4> {
  //this is slightly different from forest_edges because the town edge
  //logic for monsters is one-way
  
  // right up left down (matching dir4.index)
  let mut candidates: Vec<Dir4> = Vec::new();
  let tile: Tile = board[*pos];
  for ix in 0..4 {
    let dir: Dir4 = Dir4::list()[ix];
    let edge = tile.contents[ix];
    if edge == Terrain::Town {
      candidates.push(dir);
    }
  }
  candidates
}

pub fn eligible_for_quest(enemies: &WrapMap<Enemy>,
                          quests: &WrapMap<Quest>,
                          rng: &mut Rng) -> Option<Quest> {
  // we are eligible for a quest if
  // 1. there is an enemy type that doesn't have a quest yet
  // 2. there is at least 1 of that enemy already on the map
  let mut nme_types: Set<EnemyType> = Set::new();
  let mut nme_counts: Map<EnemyType, u16> = Map::new();

  // initialize
  for nme_t in EnemyType::list()[0..3].iter() {
    nme_types.insert(*nme_t);
    nme_counts.insert(*nme_t, 0);
  }

  // remove enemy types that already have a quest
  for (_, quest) in quests.iter() {
    nme_types.remove(&quest.target);
  }

  // remove enemy types that don't have any spawned enemies
  for (_, nme) in enemies.iter() {
    let count = nme_counts[&nme.t];
    nme_counts.insert(nme.t, count + 1);
  }
  for (nme_t, count) in nme_counts.iter() {
    if *count == 0 {
      nme_types.remove(nme_t);
    }
  }

  if nme_types.len() > 0 {
    let selected_type = nme_types[rng.next_u32() as usize % nme_types.len()];
    let mut quest = Quest::new();
    let quota = MIN_QUEST + rng.next_u64() % (MAX_QUEST - MIN_QUEST + 1);
    quest.target = selected_type;
    quest.quota = quota;
    Some(quest)
  } else {
    None
  }
}
