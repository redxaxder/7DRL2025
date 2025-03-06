#![allow(dead_code)]

use rl2025::*;
use rl2025::tiles::boss_lair;
use footguns::Ref;

type RegionId = u16;
const MONSTER_SPAWN_CHANCE: u64 = 20; // units are 1/10 percent
const QUEST_SPAWN_CHANCE: u64 = 70; // units are 1/10 percent, roughly once in 15 tiles
const REGION_REWARD_THRESHOLD: usize = 4;
const NUM_BOSSES: usize = 15;
const QUEST_REWARD: i64 = 5;

const STARTING_HP: i64 = 5;
const STARTING_TILES: i64 = 30;

const DEBUG_IMMORTAL: bool = false;
const BOSS_LOCATION:IVec = IVec::ZERO;


//const MONSTER_COLOR: Color = PURPLE;
const MONSTER_COLOR: Color = Color{r: 0.88, g:0.28, b: 0.7, a: 1.};

struct SimulationState {
  player_pos: Position,
  player_hp: i64,
  player_hp_max: i64,
  player_xp: i64,
  player_level: i64,
  player_tiles: i64,
  player_next_tile: Tile,
  player_defeat: bool,
  next_quest: Option<Quest>,
  player_tile_transform: D8,
  monster_turns: i64,

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
  num_bosses: usize,
  rng: Rng,

  quests: WrapMap<Quest>,
  prizes: WrapMap<Prize>,

  player_dmap: DMap,
  nearest_enemy_dmap: DMap,


  // Animation stuff
  animations: AnimationQueue,
  ragdolls: Map<UnitId, Ref<Ragdoll>>,
  particles: Vec<Ref<Particle>>,
  hud: Ref<Hud>,

  // record where stuff gets drawn in ui
  layout: Map<HudItem, Rect>,
}

pub struct Hud {
  pub xp: i64,
  pub hp: i64,
  pub hp_color: Color,
  pub tiles: i64,
  pub turns: i64,
  pub defeat: bool,
  pub bosses: usize,
}
impl Hud {
  pub fn new() -> Self {
    Self {
      xp: 0,
      hp: STARTING_HP,
      hp_color: WHITE,
      tiles: STARTING_TILES,
      turns: 0,
      defeat: false,
      bosses: NUM_BOSSES,
    }
  }
}



#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum HudItem{
  Hp, Xp, Tile, SpeedPenalty, Bar
}

pub struct Ragdoll {
  pub pos: Vec2,
  pub img: Img,
  pub color: Color,
  pub dead: bool,
}
pub type Particle = Ragdoll;

impl SimulationState {
  pub fn new() -> Self {
    let mut sim = SimulationState {
      player_pos: IVec::ONE,
      player_hp: STARTING_HP,
      player_hp_max: STARTING_HP,
      player_xp: 0,
      player_level: 1,
      player_tiles: STARTING_TILES,
      player_next_tile: Tile::default(),
      player_defeat: false,
      next_quest: None,
      player_tile_transform: D8::E,
      monster_turns: 0,
      board: Buffer2D::new(Tile::default(), BOARD_RECT),
      enemies: WrapMap::new(BOARD_RECT),
      quests: WrapMap::new(BOARD_RECT),
      prizes: WrapMap::new(BOARD_RECT),
      regions: Buffer2D::new([RegionId::MAX;4], BOARD_RECT),
      next_region_id: 1,
      open_regions: Set::new(),
      void_frontier: Set::new(),
      region_sizes: Map::new(),
      region_start: Map::new(),
      rng: from_current_time(),
      player_dmap: Buffer2D::new(0, BOARD_RECT),
      nearest_enemy_dmap: Buffer2D::new(0, BOARD_RECT),
      num_bosses: NUM_BOSSES,

      // Animation stuff
      animations: AnimationQueue::new(),
      ragdolls: Map::new(),
      particles: Vec::new(),
      hud: Ref::new(Hud::new()),

      layout: Map::new(),
    };
    let boss_lair_tiles = boss_lair(&mut sim.rng);
    sim.place_tile(Position { x: -1, y: 1 }, boss_lair_tiles[0]);
    sim.place_tile(Position { x: 0, y: 1 }, boss_lair_tiles[1]);
    sim.place_tile(Position { x: 1, y: 1 }, boss_lair_tiles[2]);
    sim.place_tile(Position { x: -1, y: 0 }, boss_lair_tiles[3]);
    sim.place_tile(Position { x: 0, y: 0 }, boss_lair_tiles[4]);
    sim.place_tile(Position { x: 1, y: 0 }, boss_lair_tiles[5]);
    sim.place_tile(Position { x: -1, y: -1 }, boss_lair_tiles[6]);
    sim.place_tile(Position { x: 0, y: -1 }, boss_lair_tiles[7]);
    sim.place_tile(Position { x: 1, y: -1 }, boss_lair_tiles[8]);
    sim.spawn_enemy(EnemyType::GhostWitch, BOSS_LOCATION);
    sim.move_player(sim.player_pos);
    sim.next_tile();

    sim
  }

  pub fn spawn_enemy(&mut self, t: EnemyType, at: Position) {
    let nme = Enemy::new(t);
    self.enemies.insert(at, nme);
    let rdr = self.ragdoll_ref(nme.id);
    if self.board[at] != Tile::default() {
      unsafe {
        rdr.get().img = enemy_img(nme.t);
        rdr.get().color = MONSTER_COLOR;
      }
    }
  }

  pub fn player_level_up(&mut self) {
    if self.player_xp < self.player_xp_next() { return; }
    self.add_xp(-self.player_xp_next());
    self.player_hp_max += 1;
    self.full_heal();
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

      let neighbors = subtile_neighbors((p,d));
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

      // place quest
      if let Some(_) = self.next_quest {
        self.quests.insert(position, self.next_quest.take().unwrap());
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

  pub fn reward_completed_region(&mut self, rid: RegionId, display: &Display) {
    let (position, dir) = self.region_start[&rid];
    let terrain = self.board[position].contents[dir.index()];
    let size = self.region_sizes[&rid];
    if terrain == Terrain::River {
      // Cancel the reward if the region is a river without
      // a source
      let mut river_reward = false;
      let mut frontier: Vec<(Position, Dir4)> = vec![(position,dir)];
      let mut visited: Set<(Position, Dir4)>  = Set::new();
      while let Some(subtile@(tile,_)) = frontier.pop() {
        if self.board[tile].count(Terrain::River) == 1 {
          river_reward = true;
          break;
        }
        if !visited.contains(&subtile) {
          visited.insert(subtile);
          let neighbors = subtile_neighbors(subtile);
          for i in 0..4 {
            let n@(p,d) = neighbors[i];
            if i == 0 &&
              self.board[p].contents[4] != Terrain::River {
                continue;
            }
            if self.board[p]
              .contents[d.index()] == Terrain::River {
              frontier.push(n);
            }
          }
        }
      }
      if !river_reward { return; }
    }
    let xp_reward = size.saturating_sub(REGION_REWARD_THRESHOLD);
    if xp_reward > 0 {
      let from = display.pos_rect(self.player_pos.into()).center();
      {
        let to = self.layout[&HudItem::Xp].center();
        for i in 0..xp_reward {
          let delay = i as f64 * 0.1;
          self.animations.append_empty(0.).require(PLAYER_UNIT_ID);
          self.animations.append_empty(delay).chain();
          self.launch_particle(from, to, XP, YELLOW, 3., 0.03)
            .chain();
            self.add_xp(1).chain();
        }
      }
      let to = self.layout[&HudItem::Tile].center();
      self.animations.append_empty(0.).require(PLAYER_UNIT_ID);
      self.launch_particle(from, to, TILE, GRAY, 3., 0.1).chain();
      self.add_tiles(1).chain();
      //debug!("region reward: {} xp 1 tile", xp_reward);
    }
  }

  pub fn player_current_tile(&self) -> Tile {
    self.player_tile_transform * self.player_next_tile
  }

  pub fn next_tile(&mut self) {
    self.player_next_tile = tiles::generate(&mut self.rng);
    self.player_tile_transform = D8::E;
    self.add_tiles(-1);

    // does the next tile have a quest?
    if roll_chance(&mut self.rng, QUEST_SPAWN_CHANCE) {
      debug!("quest");
      if let Some(quest) = eligible_for_quest(&self.enemies, &self.quests, &mut self.rng) {
        debug!("{:?}", quest);
        self.next_quest = Some(quest);
      }
    }
    //debug!("nq {:?}", self.next_quest);
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

  pub fn move_enemy(&mut self, from: Position, to: Position, speed: f64) {
    info!("move enemy {:?} -> {:?}", from, to);
    if !self.enemies.contains_key(to) {
      if let Some(nme) = self.enemies.remove(from) {
        self.enemies.insert(to, nme);
        let t0 = Vec2::from(from);
        let t1 = Vec2::from(to);
        let mid = (t0 + t1) / 2.;
        self.animations.append(empty_animation)
          .reserve([from,to])
          .reserve(nme.id);
        self.animate_unit_motion(nme.id, t0, mid, 0.2 / speed)
          .chain();
        if self.board[from] == Tile::default() {
          let rgr = self.ragdoll_ref(nme.id).clone();
          self.animations.append(move |_time| {
            unsafe {
              let ragdoll = rgr.get();
              ragdoll.img = enemy_img(nme.t);
              ragdoll.color = MONSTER_COLOR;
            }
            false
          }).chain();
        }
        self.animate_unit_motion(nme.id, mid, t1, 0.2 / speed)
          .chain()
          .reserve(to)
          .reserve(nme.id);
      }
    }
  }

  pub fn player_relative_coordinates(&self, p: Vec2) -> Vec2 {
    let w = BOARD_RECT.width as f32;
    let h = BOARD_RECT.height as f32;
    let r: Rect = Rect {
      x: self.player_pos.x as f32 - (w / 2.),
      y: self.player_pos.y as f32 - (h / 2.),
      w, h
    };
    wrap_rect(r, p)
  }

  pub fn slay_enemy(&mut self, at: Position, dir: Dir4, display: &Display) {
    let Some(nme) = self.enemies.remove(at) else { return; };
    // credit quests
    for (_, quest) in self.quests.iter_mut() {
      if quest.target == nme.t && quest.quota > 0 {
        quest.quota -= 1;
      }
    }

    // do animation
    let id = nme.id;
    let mut velocity: Vec2 = Vec2::from(dir) * 3.;
    velocity.x += (self.rng.next_u32() % 1000) as f32 / 1000.;
    velocity.y += (self.rng.next_u32() % 1000) as f32 / 1000.;
    velocity *= 8.;
    self.animate_unit_fling(id, at.into(), velocity, 0.2).require(id);
    self.add_hp(-1).require(id);
    self.animations.append(empty_animation)
      .require([id, PLAYER_UNIT_ID]);
    self.launch_particle(
      display.pos_rect(Vec2::from(at)).center(),
      self.layout[&HudItem::Xp].center(),
      XP,
      YELLOW,
      3.,
      0.03,
    ).chain();
    self.add_xp(1).chain();
  }

  pub fn add_xp(&mut self, amount: i64) -> &mut Animation {
    self.player_xp += amount;
    let hud = self.hud.clone();
    self.animations.append(move |_| unsafe {
      hud.get().xp += amount;
      false
    })
  }

  pub fn full_heal(&mut self) -> &mut Animation {
    self.add_hp(self.player_hp_max - self.player_hp)
  }
  pub fn add_hp(&mut self, amount: i64) -> &mut Animation {
    let is_damage = amount < 0;
    self.player_hp += amount;
    let hudref = self.hud.clone();
    let duration = 0.1;
    self.animations.append(move |time| unsafe {
      let hud = hudref.get();
      if is_damage { hud.hp_color = RED; }
      let more = time.progress(duration) < 1.;
      if !more {
        hud.hp_color = WHITE;
        hud.hp += amount;
      }
      more
    })
  }

  pub fn add_monster_turns(&mut self, amount: i64) -> &mut Animation {
    self.monster_turns += amount;
    let hudref = self.hud.clone();
    self.animations.append(move |_| unsafe {
      let hud = hudref.get();
      hud.turns += amount;
      false
    })
  }

  pub fn in_combat(&mut self) -> bool {
    let mut in_combat = false;
    for d in Dir4::list() {
      let adj = self.player_pos + d.into();
      // monsters in void don't count
      if self.board[adj] == Tile::default() { continue; }
      in_combat = in_combat || self.enemies.get(adj).is_some();
    }
    in_combat
  }

  pub fn is_road_dir(&self, dir: Dir4) -> bool {
    // two cases:
    // 1) there is an existing road here we can take
    // 2) there is a half road here, with the other half
    //    in hand and oriented the right way
    // either way, the check for the first half of the road is the same
    let target = self.player_pos + dir.into();
    let opp = dir.opposite();
    let first_half = Terrain::Road ==
      self.board[self.player_pos].contents[dir.index()];

    let second_half = Terrain::Road ==
      if self.board[target] == Tile::default() {
        self.player_current_tile().contents[opp.index()]
      } else {
        self.board[target].contents[opp.index()]
      };
    first_half && second_half
  }


  pub fn add_tiles(&mut self, amount: i64) -> &mut Animation {
    self.player_tiles += amount;
    let hudref = self.hud.clone();
    self.animations.append(move |_| unsafe {
      hudref.get().tiles += amount;
      false
    }).chain()

  }

  fn ragdoll_ref(&mut self, unit_id: UnitId) -> Ref<Ragdoll> {
    if let Some(rgr) = self.ragdolls.get(&unit_id) {
      (*rgr).clone()
    } else if unit_id == PLAYER_UNIT_ID {
      let rgr = Ref::new(Ragdoll {
        pos: self.player_relative_coordinates(Vec2::from(self.player_pos)),
        color: WHITE,
        img: HERO,
        dead: false,
      });
      self.ragdolls.insert(unit_id, rgr.clone());
      rgr
    } else {
      let Some((&pos, _nme)) = self.enemies.iter().find(|(_pos,nme)| { nme.id == unit_id }) else {
        panic!("tried to generate ragdoll for an id that doesn't exist")
      };
      let rgr = Ref::new(Ragdoll {
        pos: self.player_relative_coordinates(Vec2::from(pos)),
        color: Color{a: 0., ..RED},
        img: UNKNOWN_ENEMY,
        dead: false,
      });
      self.ragdolls.insert(unit_id, rgr.clone());
      let result = rgr.clone();
      self.animations.append(move |_| unsafe {
        rgr.get().color.a = 1.;
        false
      }).reserve(unit_id).reserve(pos);
      result
    }
  }

  pub fn animate_unit_fling(&mut self, u: UnitId, p0: Vec2, velocity: Vec2, duration: Seconds) -> &mut Animation {
    let uref = self.ragdoll_ref(u);
    self.animations.append(move |time| {
      let progress = time.progress(duration);
      unsafe {
        uref.get().pos = p0 + velocity * (time.elapsed as f32);
        if progress >= 1. {
          uref.get().dead = true;
        }
      }
      progress < 1.
    })
  }

  pub fn animate_unit_motion(&mut self, u: UnitId, p0: Vec2, p1: Vec2, duration: Seconds) -> &mut Animation {
    let prc0 = self.player_relative_coordinates(p0);
    let prc1 = self.player_relative_coordinates(p1);
    let uref = self.ragdoll_ref(u);
    self.animations.append(move |time| {
      let c = time.progress(duration);
        unsafe {
          uref.get().pos = c * prc1 + (1.-c) * prc0;
        }
      c < 1.
    })
  }

  pub fn launch_particle(
    &mut self,
    from: ScreenCoords,
    to: ScreenCoords,
    img: Img,
    color: Color,
    kick: f64, // multiplier on initial (random) velocity
    decay: f64 // percentage of remaining distance remaining after a second
    ) -> &mut Animation {
    let p = Ref::new(Particle {
      pos: from, img,
      color: INVISIBLE,
      dead: false
    });

    let v: Ref<Vec2> = Ref::new({
      let mut x: f32 = ((self.rng.next_u32() as i32 % 11) - 5) as f32;
      x = x.signum() * x.abs().sqrt();
      let mut y = ((self.rng.next_u32() as i32 % 11) - 5) as f32;
      y = y.signum() * y.abs().sqrt();
      kick as f32 * Vec2 { x, y }
    });

    self.particles.push(p.clone());
    debug!("animations: {}", self.animations.len());
    debug!("particles: {}", self.particles.len());

    self.animations.append(move |time: Time| {
      let d = decay.powf(time.delta) as f32;
      let offset = p.pos - to;
      unsafe{
        let it = p.get();
        it.pos = to + d * offset;
        it.pos += *v;
        *v.get() *= d;
        it.color = color;
        it.dead = to.distance(it.pos) < 20.;
      }
      !p.dead
    })
  }

  pub fn move_player(&mut self, to: Position) {
    let from = self.player_pos;
    self.player_pos = to;
    self.animate_unit_motion(PLAYER_UNIT_ID, from.into(), to.into(), 0.3)
      .reserve([from,to])
      .reserve(PLAYER_UNIT_ID);
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

  pub fn tick_animations(&mut self) {
    self.animations.tick();
    let mut died = vec!();
    for (id,v) in self.ragdolls.iter() {
      if v.dead { died.push(*id); }
    }
    for dead in died {
      self.ragdolls.remove(&dead);
    }
    for i in (0.. self.particles.len()).rev() {
      if self.particles[i].dead {
        self.particles.remove(i);
      }
    }
  }
}



#[macroquad::main("7drl")]
async fn main() {
  debug!("This is a debug message");
  info!("and info message");
  error!("and errors, the red ones!");
  warn!("Or warnings, the yellow ones.");

  let mut sim = SimulationState::new();

  let mut resources = Resources::new(ASSETS);
  for path in LOAD_ME { resources.load_texture(path, FilterMode::Linear); }

  let display_dim: Vec2 = DISPLAY_GRID.dim();
  let mut display = Display::new(resources, display_dim);

  let mut victory = false;

  loop {
    if get_keys_pressed().len() > 0 {
      sim.animations.hurry(2.);
    }

    let mut inputdir: Option<Dir4> = None;

    if let Some(input) = get_input() {
      if sim.hud.defeat {
        sim = SimulationState::new();
        continue;
      }
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
          if sim.player_tiles > 0 {
            sim.next_quest = None;
            sim.next_tile();
          }
          
        }
        Input::LevelUp => {
          sim.player_level_up()
        }
      }
    }

    let mut tile_placed: bool = false;
    let mut player_moved: bool = false;
    let mut needs_road = false;
    let mut can_move = true;

    if let Some(playermove) = inputdir  {
      let target = sim.player_pos + playermove.into();
      let target_empty = sim.board[target] == Tile::default();

      // do combat
      if sim.in_combat() {
        let mut defeated_boss = false;
        if let Some(Enemy { t: EnemyType::GhostWitch, .. }) = sim.enemies.get(target) {
          let mut speed_mul: f64 = 1.;
          let mut delay = 0.;
          while sim.num_bosses > 0 {
          //for _ in 0..NUM_BOSSES-1 {
            let id = sim.enemies.get(target).unwrap().id;
            delay += 0.3/speed_mul;
            sim.animations.append_empty(delay).reserve(id);
            sim.slay_enemy(target, playermove, &display);
            sim.num_bosses -= 1;
            let hudref = sim.hud.clone();
            sim.animations.append(move |_| unsafe {
              hudref.get().bosses -= 1;
              false
            }).reserve(id);
            sim.spawn_enemy(EnemyType::GhostWitch, target);
            speed_mul += 0.5;
            if sim.player_hp < 1 {
              break;
            }
          }
          if sim.player_hp > 0 {
            defeated_boss = true;
          }
        }
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
              if let Some(Enemy { t: EnemyType::GhostWitch, .. }) = sim.enemies.get(cursor) {
                if !defeated_boss {
                  continue;
                }
              }
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
          let mut speed_mul: f64 = 1.;
          while sim.enemies.contains_key(target) {
            if sim.player_hp < 1 { break; }
            speed_mul += 0.5;
            sim.slay_enemy(target, playermove, &display);
            // enemies behind move up
            let mut vacated = target;
            let mut dist = 0;
            'scooch: loop {
              for d in Dir4::list() {
                let neighbor = vacated + d.into();
                if let Some(&dist2) = crowd.get(&neighbor) {
                  // enemies only want to scooch closer
                  if dist2 <= dist { continue; }
                  if sim.enemies.contains_key(neighbor) {
                    sim.move_enemy(neighbor, vacated, speed_mul);
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
          victory = defeated_boss && sim.player_hp > 0;
        } else { // nobody in this spot to fight
          needs_road = true;
        }

        { // Quest reward, spawn quest items
          let mut fulfilled_quests: WrapMap<Quest> = WrapMap::new(BOARD_RECT);
          let ppos = sim.player_pos;
          for (&p, &q) in sim.quests.clone().iter() {
            if q.quota < 1 {
              fulfilled_quests.insert(p, q);
              sim.quests.remove(p);
              sim.prizes.insert(p, Prize::Heal);
              // launch particles from player pos since quest
              // origin might be offscreen.
              // maybe later check if its on screen first
              let from = display.pos_rect(Vec2::from(ppos)).center();
              let to = sim.layout[&HudItem::Tile].center();
              for i in 0..(QUEST_REWARD as u8) {
                let delay = f64::from(i) * 0.15;
                sim.animations.append_empty(0.).require(PLAYER_UNIT_ID);
                sim.animations.append_empty(delay)
                  .chain();
                sim.launch_particle(from, to, TILE, GRAY, 3., 0.1)
                  .chain();
                sim.add_tiles(1)
                  .chain();
              }
            }
          }
          sim.animations.append_empty(0.).require(ppos);
          for p in fulfilled_quests.keys() {
            sim.quests.remove(*p);
          }
        }

      }
      let using_road = sim.is_road_dir(playermove);
      can_move = can_move && (!needs_road || using_road);
      can_move = can_move && (!target_empty || sim.tile_compatibility(target, sim.player_current_tile()) > 0);
      if sim.player_tiles < 1  && sim.board[target] == Tile::default() {
        can_move = false;
      }
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

        sim.move_player(target);
        player_moved = true;
        //debug!("player: {:?}", sim.player_pos);

        // try to collect prize
        if let Some(&prize) = sim.prizes.get(target) {
          sim.prizes.remove(target);
          let from = display.pos_rect(target.into()).center();
          let to = sim.layout[&HudItem::Hp].center();
          sim.animations.append_empty(0.).reserve(PLAYER_UNIT_ID);
          sim.launch_particle(from, to,
            prize_img(prize), RED,
            3., 0.02
          ).chain();
          sim.full_heal().chain();
        }

        // try to place tile
        if sim.board[sim.player_pos] == Tile::default() && sim.player_tiles > 0 {
          sim.place_tile(sim.player_pos, sim.player_current_tile());
          sim.next_tile();
          tile_placed = true;
          // new tiles smoosh monsters
          if let Some(nme) = sim.enemies.remove(sim.player_pos) {
            sim.ragdolls.remove(&nme.id);
          }
          sim.update_region_sizes();


          { // check for perfect tile bonuses
            // on placed tile and neighbors
            let mut to_check = vec!(sim.player_pos);
            for d in Dir4::list() {
              to_check.push(sim.player_pos + d.into());
            }
            for &p in &to_check {
              let mut is_matched = true;
              let ptile = sim.board[p];

              for d in Dir4::list() {
                let ntile = sim.board[p + d.into()];
                if ptile.contents[d.index()]
                  != ntile.contents[d.opposite().index()] {
                    is_matched = false;
                }
              }
              if is_matched {
                let from = display.pos_rect(p.into()).center();
                let to = sim.layout[&HudItem::Tile].center();
                sim.animations.append_empty(0.).require(PLAYER_UNIT_ID);
                sim.launch_particle(from, to, TILE, GRAY, 3., 0.1).chain();
                sim.add_tiles(1).chain();
                debug!("perfect tile bonus");
              }
            }
          }

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
            sim.reward_completed_region(regionid, &display);
          }
        } else { // we stepped on an existing tile
          if (target_is_slow || edge_is_slow) && !using_road {
            let from = display.pos_rect(target.into()).center();
            let to = sim.layout[&HudItem::SpeedPenalty].center();
            sim.animations.append(empty_animation).require(target);
            sim.launch_particle(from, to,
              TIME, BLUE, 0.4, 0.03
            ).chain();
            sim.add_monster_turns(1).chain();
          }
        }
      }
      if sim.player_hp < 1 && !DEBUG_IMMORTAL {
        sim.player_defeat = true;

        let dirvec: Vec2 = (sim.player_pos - target).into();
        let mut velocity: Vec2 = Vec2::from(dirvec) * 3.;
        velocity.x += (sim.rng.next_u32() % 1000) as f32 / 1000.;
        velocity.y += (sim.rng.next_u32() % 1000) as f32 / 1000.;
        velocity *= 2.;
        sim.animate_unit_fling(
          PLAYER_UNIT_ID,
          sim.player_pos.into(),
          velocity,
          2.).reserve(PLAYER_UNIT_ID);
        let hudref = sim.hud.clone();
        sim.animations.append(move |_| unsafe {
          hudref.get().defeat = true;
          false
        }).chain();
      }
    }

    //debug!("{:?}", sim.player_pos);
    let camera_offset: IVec = display.camera_focus - sim.player_pos;
    display.camera_focus = sim.player_pos + CAMERA_TETHER.clamp_pos(camera_offset);

    {//monsters
      let mut monsters_go = false;
      if tile_placed || (player_moved && sim.player_tiles < 1) {
        sim.animations.sync_positions();
        sim.add_monster_turns(1).chain();
        monsters_go = true;
        sim.update_player_dmap();
      }
      let mut spawns = vec!();

      let mut acceleration = 1.0;
      while monsters_go && sim.monster_turns > 0 {
        spawns.clear();
        sim.update_nearest_dmap();
        //do monster turn
        for (pos, _) in sim.enemies.clone().iter() {
          let maybe_pos = enemy_pathfind(&mut sim, *pos);
          if let Some(new_pos) = maybe_pos {
            sim.move_enemy(*pos, new_pos, acceleration);
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
            spawns.push((random_enemy_type,p));
            //debug!("spawned a monster {:?} at {:?}", nme.t, p)
          }
        }
        for(t,p) in &spawns {
          sim.spawn_enemy(*t,*p);
        }

        sim.animations.append_empty(0.3 / acceleration).chain();
        sim.animations.sync_positions().chain();
        sim.add_monster_turns(-1).chain();
        acceleration += 0.5;
      }
    }

    sim.tick_animations();


    let scale: f32 = f32::min(
      screen_width() / display.dim.x as f32,
      screen_height() / display.dim.y as f32,
    );

    const DRAW_BOUNDS:IRect = IRect{ x: -8, y:-8, width: 17, height: 17};
    { // Redraw the display
      set_camera(&display.render_to);
      clear_background(DARKPURPLE);


      // DEBUG GRID VERTICES
      //let spots = IRect{x: 0, y: 0, width: 20, height: 20};
      //for s in spots.iter() {
      //  let v = Vec2::from(s) * 128.;
      //  draw_circle(v.x, v.y, 20., BLUE);
      //}

      // Draw tile backgrounds
      for offset in DRAW_BOUNDS.iter() {
        let p = sim.player_pos + offset;
        if sim.board[p] == Tile::default() { continue; }
        let r = display.pos_rect(p.into());
        draw_rectangle(r.x, r.y, r.w, r.h, DARKBROWN);
        //display.draw_tile_1(r, tile, terrain);
      }
      // draw terrain
      for &terrain in Terrain::DRAW_ORDER {
        for offset in DRAW_BOUNDS.iter() {
          let p = sim.player_pos + offset;
          let tile = sim.board[p];
          let r = display.pos_rect(p.into());
          display.draw_tile_1(r, tile, terrain);
        }
      }
      for offset in DRAW_BOUNDS.iter() { // draw quests and prized
        let p = sim.player_pos + offset;
        let r = display.pos_rect(p.into());
        if sim.quests.contains_key(p) {
          let quest = sim.quests[p];
          draw_quest(&display, &r, &quest);
        }
        if let Some(prize) = sim.prizes.get(p) {
          let img = prize_img(*prize);
          display.draw_img(r, RED, &img);
        }
      }

      for offset in DRAW_BOUNDS.iter() { // blocked tile hints
        let p = sim.player_pos + offset;
        let mut blocked = false;
        let mut blocked_color = BLACK;
        // if the target is in the frontier
        // and the current tile cant fit there (in current orientation), it is blocked


        if sim.void_frontier.contains(&BOARD_RECT.wrap(p)) {
          if sim.tile_compatibility(p, sim.player_current_tile()) == 0  || sim.player_tiles < 1 {
            blocked = true;
            blocked_color = GRAY;
          }
        }

        if let Ok(d) = Dir4::try_from(offset) {
          // we're locked in combat, and this is not a road direction
          if sim.in_combat() && !sim.is_road_dir(d) {
            // we can't step on void
            blocked = blocked || sim.board[p] == Tile::default();
            // we can't step on a free space
            blocked = blocked || !sim.enemies.contains_key(p);
          }
        }

        if blocked {
          display.draw_grid( p.into(), blocked_color, &BLOCKED);
        }
      }

      // draw enemies
      for ragdoll in sim.ragdolls.values() {
        display.draw_grid(
          ragdoll.pos,
          ragdoll.color,
          &ragdoll.img
        );
      }

      // draw boss count
      for offset in DRAW_BOUNDS.iter() { // draw quests and prized
        let p = sim.player_pos + offset;
        if p != BOSS_LOCATION { continue; }
        let r = display.pos_rect(p.into());
        let text = format!("{}", sim.hud.bosses);
        let font_size = 70;
        if sim.hud.bosses >= 2 {
          let metrics = measure_text(&text, None, font_size, 1.);
          let leftover = r.w - metrics.width;

          draw_text(&text, r.x + 0.5 * leftover,
            r.y + metrics.offset_y - (r.h * 0.15),
            font_size as f32, MONSTER_COLOR
            );

        }
      }

      { // draw HUD
        let font_size = 60;
        let font_scale = 1.;

        let margin = 15.;
        let sz = DISPLAY_GRID.full_tile_size();

        { // Bar
          let x = 0.;
          let h = sz.y + 2. * margin;
          let y = display.dim.y - h;
          let w = display.dim.x;
          let rect = Rect { x, y, w, h };
          sim.layout.insert(HudItem::Bar, rect);
          draw_rectangle(x, y, w, h, DARKGRAY);
        }

        if sim.hud.defeat {
            let bar = sim.layout[&HudItem::Bar];
            let display_text = format!("Defeated...");
            let textdim: TextDimensions = measure_text(&display_text, None, font_size, font_scale);
            let leftover = bar.h - textdim.height;
            let x = (display.dim.x - textdim.width - margin)/2.;
            let y = bar.y + (0.5 * leftover) + textdim.offset_y;
            draw_text(&display_text, x, y, font_size as f32, WHITE);
        } else {
          { // Next Tile
            let hudbar: Rect = sim.layout[&HudItem::Bar];
            let r = Rect {
              x: hudbar.w - sz.x - margin,
              y: hudbar.y + margin ,
              w: sz.x,
              h: sz.y
            };
            sim.layout.insert(HudItem::Tile, r);
            display.draw_tile(r, sim.player_current_tile());
            if let Some(q) = sim.next_quest {
              draw_quest(&display, &r, &q);
            }
          }

          { // Remaining tiles
            let r = sim.layout[&HudItem::Tile];
            let bar = sim.layout[&HudItem::Bar];
            let remaining_tiles = format!("{}", sim.hud.tiles);
            let textdim: TextDimensions = measure_text(&remaining_tiles, None, font_size, font_scale);
            let leftover = bar.h - textdim.height;
            let x = r.x - textdim.width - margin;
            let y = bar.y + (0.5 * leftover) + textdim.offset_y;
            draw_text(&remaining_tiles, x, y, font_size as f32, WHITE);
          }

          { // Current/Max HP and XP
            let bar = sim.layout[&HudItem::Bar];
            let hp = format!("HP: {}/{} ", sim.hud.hp, sim.player_hp_max);
            let hpdim: TextDimensions = measure_text(&hp, None, font_size, font_scale);
            let xp = format!("XP: {}/{}", sim.hud.xp, sim.player_xp_next());
            let xpdim: TextDimensions = measure_text(&xp, None, font_size, font_scale);
            let leftover = bar.h - hpdim.height - xpdim.height;
            let hpr = Rect {
              x: margin,
              w: hpdim.width,
              h: hpdim.height,
              y: bar.y + (0.33 * leftover),
            };
            let xpr = Rect {
              x: margin,
              w: xpdim.width,
              h: xpdim.height,
              y: bar.y + (0.66 * leftover) + hpr.h,
            };
            draw_text(&hp, hpr.x, hpr.y + hpdim.offset_y, font_size as f32, sim.hud.hp_color);
            draw_text(&xp, xpr.x, xpr.y + xpdim.offset_y, font_size as f32, WHITE);
            sim.layout.insert(HudItem::Hp, hpr);
            sim.layout.insert(HudItem::Xp, xpr);
          }


          { // num monster turns
            let bar = sim.layout[&HudItem::Bar];
            let icon_rect = Rect{
              x: bar.w * 0.5,
              y: bar.y + margin,
              w: sz.x,
              h: sz.y,
            };
            if sim.hud.turns > 0 {
              let text = format!("{}", sim.hud.turns);
              let textdim = measure_text(&text, None, font_size, font_scale);
              let y = bar.y + 0.5 * (bar.h - textdim.height) + textdim.offset_y;
              let x = icon_rect.x - textdim.width - margin;
              display.draw_img( icon_rect, BLUE, &TIME);
              draw_text(&text,x,y, font_size.into(), WHITE);
            }
            sim.layout.insert(HudItem::SpeedPenalty, icon_rect);
          }
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

      { // Draw particles
        for p in &sim.particles {
          let r = Rect{x:-32., y: -32., w: 64., h: 64.}.offset(p.pos);
          display.draw_img(r, p.color, &p.img);
        }
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

    next_frame().await;

    if victory && sim.animations.len() == 0 {
      break;
    }
  }

  if victory {
    clear_background(BLACK);
    loop {
      draw_text("You win!", 300., 300., 64., WHITE);
      next_frame().await;
    }
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
  // add forest edges to valid set
  let mut valid: Vec<Dir4> = forest_edges(&pos, &sim.board);
  if valid.len() == 0 {
    // no forest edges means anything is a candidate
    valid = Dir4::list().into();
  }

  let town: Vec<Dir4> = town_edges(&pos, &sim.board);

  let mut candidates: Vec<IVec> = Vec::new();
  for &d in &valid {
    let target = pos + IVec::from(d);
    // skip town edges
    if town.contains(&d) { continue; }
    // dont step on me
    if equivalent(target, sim.player_pos) { continue; }
    // no void
    if sim.board[target] == Tile::default() { continue; }
    // dont step on quest
    if sim.quests.contains_key(target) { continue; }
    // dont step on prize
    if sim.prizes.contains_key(target) { continue; }
    candidates.push(target);
  }
  if sim.board[pos] != Tile::default() {
    candidates.push(pos);
  }
  match sim.enemies[pos].t {
    EnemyType::Clyde => {}
    EnemyType::Blinky => {
      let mut min_score: i16 = i16::MAX;
      for &c in &candidates {
        min_score = min_score.min(sim.player_dmap[c]);
      }
      candidates = candidates.drain(..).filter(|c|{
        min_score == sim.player_dmap[*c]
      }).collect();
    }
    EnemyType::Pinky => {
      let mut max_score: i16 = i16::MIN;
      for &c in &candidates {
        max_score = max_score.max(sim.nearest_enemy_dmap[c]);
      }
      candidates = candidates.drain(..).filter(|c|{
        max_score == sim.nearest_enemy_dmap[*c]
      }).collect();
    }
    EnemyType::GhostWitch => {
      candidates.clear();
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
  let mut nme_counts: Map<EnemyType, u64> = Map::new();

  // initialize
  for nme_t in EnemyType::list()[0..3].iter() {
    nme_counts.insert(*nme_t, 0);
  }

  // remove enemy types that already have a quest
  for (_, quest) in quests.iter() {
    nme_counts.remove(&quest.target);
  }

  // remove enemy types that don't have any spawned enemies
  for (_, nme) in enemies.iter() {
    if nme_counts.contains_key(&nme.t) {
      let count = nme_counts[&nme.t];
      nme_counts.insert(nme.t, count + 1);
    }
  }
  nme_counts = nme_counts
    .iter()
    .filter(|(_, c)| **c > 0)
    .map(|(k,v)| (*k, *v))
    .collect();
  debug!("nme_counts {:?}", nme_counts);

  if nme_counts.len() > 0 {
    let nme_types: Vec<&EnemyType> = nme_counts.keys().collect();
    let selected_type = nme_types[rng.next_u32() as usize % nme_types.len()];
    let mut quest = Quest::new();
    let quota = nme_counts.get(selected_type).unwrap();
    quest.target = *selected_type;
    quest.quota = *quota;
    Some(quest)
  } else {
    None
  }
}
