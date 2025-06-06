#![allow(dead_code)]

use rl2025::*;
use std::rc::Rc;
use rl2025::tiles::boss_lair;
use footguns::Ref;
use macroquad::audio::{load_sound_from_bytes, Sound};

type Path = &'static str;
type RegionId = u16;

// each turn, every void space produces a spawn point
// they increase monster spawn chance
// when a monster spawns, these are consumed
const MONSTER_SPAWN_POINTS: i64 = 30;
const QUEST_SPAWN_CHANCE: u64 = 83; // units are 1/10 percent, roughly once in 12 tiles
const QUEST_MIN: u64 = 3;
const FOREST_ESCAPE_CHANCE: u64 = 250;
const REGION_REWARD_THRESHOLD: i64 = 4;
const NUM_BOSSES: usize = 15;
const QUEST_REWARD: i64 = 5;

const STARTING_HP: i64 = 7;
const STARTING_TILES: i64 = 35;

const DEBUG_IMMORTAL: bool = false;
const BOSS_LOCATION:IVec = IVec::ZERO;

const BASE_ANIMATION_DURATION: f64 = 0.5;


//const MONSTER_COLOR: Color = PURPLE;
//
struct UIState {
  // Animation stuff
  animations: AnimationQueue,
  ragdolls: Map<UnitId, Ref<Ragdoll>>,
  particles: Vec<Ref<Particle>>,
  flying_tiles: Vec<Ref<AnimTile>>,
  hud: Ref<Hud>,
  camera_ref: Ref<IVec>,
  compass_flash: f32,
  // record where stuff gets drawn in ui
  layout: Map<HudItem, Rect>,


  // Audio
  sounds: Map<Path, Rc<Sound>>,
}

impl UIState {
  pub fn new(sounds: &Map<Path, Rc<Sound>>) -> Self {
    Self {

      animations: AnimationQueue::new(),
      ragdolls: Map::new(),
      particles: Vec::new(),
      flying_tiles: Vec::new(),
      hud: Ref::new(Hud::new()),
      camera_ref: Ref::new(IVec::ZERO),
      compass_flash: 0.,

      layout: Map::new(),

      sounds: sounds.clone(),

    }


  }
}

struct SealedState {
  player_immortal: bool,
  player_next_tile: Tile,
  next_quest: Option<Quest>,
  score_tiles_placed:  i64,
  board: Buffer2D<Tile>,
  regions: Buffer2D<[RegionId;4]>,
  region_sizes: Map<RegionId, i64>,
  // the subposition of the first tile in this region
  region_start: Map<RegionId, Subposition>,
  next_region_id: RegionId,
  // regions that border void
  open_regions: Set<RegionId>,
  // positions bordering void
  void_frontier: WrapSet,
  enemy_supply: i64,

  // undoable but why
  player_dmap: DMap,
  nearest_enemy_dmap: DMap,
  player_tile_transform: D8,
}

impl SealedState {
  pub fn new() -> Self {
    Self {
      player_next_tile: Tile::default(),
      player_immortal: std::env::var("IMMORTAL").is_ok() || DEBUG_IMMORTAL,
      next_quest: None,
      player_tile_transform: D8::E,
      board: Buffer2D::new(Tile::default(), BOARD_RECT),
      enemy_supply: 0,
      regions: Buffer2D::new([RegionId::MAX;4], BOARD_RECT),
      next_region_id: 1,
      open_regions: Set::new(),
      void_frontier: WrapSet::new(BOARD_RECT),
      region_sizes: Map::new(),
      region_start: Map::new(),
      player_dmap: Buffer2D::new(0, BOARD_RECT),
      nearest_enemy_dmap: Buffer2D::new(0, BOARD_RECT),
      score_tiles_placed: 0,
    }
  }
}

struct Snapshot {
  player_hp: i64,
  player_hp_max: i64,
  player_xp: i64,
  player_level: i64,
  player_tiles: i64,
  player_defeat: bool,
  monster_turns: i64,
  score_min_hp: i64,
  enemies: WrapMap<Enemy>,
  num_bosses: usize,
  rng: Rng,
  quests: WrapMap<Quest>,
  prizes: WrapMap<Prize>
}


struct Snapshots{
  saved: Vec<(Snapshot, Vec<Position>)>,
  current: Snapshot,
  player_pos: Position,
}
impl Snapshots {
  pub fn pos_mut(&mut self) -> (&Snapshot, DropBear<Position>) {
    let snapshot = &self.current;
    let ppos = &mut self.player_pos;
    let saved = &mut self.saved;
    let ptr = DropBear::new(ppos, |p| { 
      let last = &mut saved.last_mut().unwrap();
      if p != last.1.last().unwrap() {
        last.1.push(*p);
      }
    });
    (snapshot, ptr)
  }

  pub fn snapshot_mut(&mut self) -> (DropBear<Snapshot>, &Position) {
    todo!()
  }

  pub fn pos(&self) -> &Position {
    todo!()
  }
  pub fn snapshot(&self) -> &Snapshot {
    todo!()
  }

}





struct SimulationState {
  player_pos: Position,
  player_hp: i64,
  player_hp_max: i64,
  player_xp: i64,
  player_level: i64,
  player_tiles: i64,
  player_defeat: bool,
  monster_turns: i64,
  score_min_hp: i64,
  enemies: WrapMap<Enemy>,
  num_bosses: usize,
  rng: Rng,
  quests: WrapMap<Quest>,
  prizes: WrapMap<Prize>,

  ui: UIState,
  sealed: SealedState,
}

pub struct Hud {
  pub xp: i64,
  pub hp: i64,
  pub hp_color: Color,
  pub hint_color: Color,
  pub tiles: i64,
  pub turns: i64,
  pub defeat: bool,
  pub victory: bool,
  pub bosses: usize,
  pub tile_rotation: f32,
  pub tile_transform: D8,
  pub highlighted_spaces: WrapSet,
  pub hidden_spaces: WrapSet,
  pub desire_path: Vec<Position>,
}
impl Hud {
  pub fn new() -> Self {
    Self {
      xp: 0,
      hp: STARTING_HP,
      hp_color: WHITE,
      hint_color: YELLOW,
      tiles: STARTING_TILES,
      turns: 0,
      defeat: false,
      victory: false,
      bosses: NUM_BOSSES,
      tile_rotation: 0.,
      tile_transform: D8::E,
      highlighted_spaces: WrapSet::new(BOARD_RECT),
      hidden_spaces: WrapSet::new(BOARD_RECT),
      desire_path: Vec::new(),
    }
  }
}



#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum HudItem{
  Hp, Xp, Tile, SpeedPenalty, Bar, Arrows, DiscardHint, LevelHint
}

pub struct Ragdoll {
  pub pos: Vec2,
  pub img: Img,
  pub color: Color,
  pub dead: bool,
}
pub type Particle = Ragdoll;

pub struct AnimTile {
  pub pos: Vec2,
  pub tile: Tile,
  pub dead: bool,
}

impl SimulationState {
  pub fn new(sounds: &Map<Path, Rc<Sound>>) -> Self {
    let mut sim = SimulationState {
      player_pos: IVec::ONE,
      player_hp: STARTING_HP,
      player_hp_max: STARTING_HP,
      player_xp: 0,
      player_level: 1,
      player_tiles: STARTING_TILES,
      player_defeat: false,
      monster_turns: 0,
      enemies: WrapMap::new(BOARD_RECT),
      quests: WrapMap::new(BOARD_RECT),
      prizes: WrapMap::new(BOARD_RECT),
      rng: from_current_time(),
      num_bosses: NUM_BOSSES,

      sealed: SealedState::new(),
      ui: UIState::new(sounds),
      // score
      score_min_hp: STARTING_HP,
    };


    // initialize starting tiles
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
    sim.ragdoll_ref(PLAYER_UNIT_ID);
    sim.next_tile();

    sim.ui.hud.desire_path.push(sim.player_pos);


    sim
  }

  pub fn player_dead(&self) -> bool {
    self.player_hp < 1 && !self.sealed.player_immortal
  }

  pub fn transform_tile(&mut self, g: D8) {
    let duration = 0.5;
    let hudref = self.ui.hud.clone();
    let r = (g * Dir4::Right).radians();
    let t = g * self.sealed.player_tile_transform;
    self.ui.animations.append(move |time| {
      let p = time.progress(duration);
      unsafe {
        hudref.get().tile_rotation = r * p;
      }
      p < 1.
    }).reserve(PLAYER_UNIT_ID);
    self.defer_set_hud(move |hud| {
      hud.tile_transform = t;
      hud.tile_rotation = 0.;
    }).reserve(PLAYER_UNIT_ID);
    self.sealed.player_tile_transform = g * self.sealed.player_tile_transform;

  }

  pub fn defer_set_hud(&mut self, mut f: impl FnMut(&mut Hud) + 'static)
    -> &mut Animation {
    let hudref = self.ui.hud.clone();
    self.ui.animations.append(move |_| unsafe {
      (f)(hudref.get());
      false
    })
  }

  pub fn defer_play_sound(&mut self, soundpath: Path) -> &mut Animation {
    let sound = self.ui.sounds[soundpath].clone();
    self.ui.animations.append(move |_| {
      play_sound(sound.clone());
      false
    })
  }

  pub fn spawn_enemy(&mut self, t: EnemyType, at: Position) {
    let nme = Enemy::new(t);
    self.sealed.enemy_supply -= MONSTER_SPAWN_POINTS;
    self.enemies.insert(at, nme);
    let rdr = self.ragdoll_ref(nme.id);
    if self.sealed.board[at] != Tile::default() {
      unsafe {
        rdr.get().img = enemy_img(nme.t, false);
        rdr.get().color = MONSTER_COLOR;
      }
    }
  }

  pub fn calculate_crowd(&self, target: Position) -> Map<Position,u8> {
    // calculates a crowd of enemies that fight together
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
        if !self.enemies.contains_key(cursor) { continue; }
        if self.sealed.board[cursor] == Tile::default() { continue; }
        if let Some(Enemy { t: EnemyType::GhostWitch, .. }) = &self.enemies.get(cursor) {
          if distance > 0 { continue; }
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
  }


  pub fn set_enemy_alerts(&mut self, alerted: bool)  {
    for d in Dir4::list() {
      let neighbor = self.player_pos + d.into();
      if self.sealed.board[neighbor] == Tile::default() {
        continue;
      }
      let crowd = self.calculate_crowd(neighbor);
      for &pos in crowd.keys() {
        let nme = self.enemies[pos];
        let rgr = self.ragdoll_ref(nme.id);
        self.ui.animations.append(move |_| unsafe {
          rgr.get().img = enemy_img(nme.t, alerted);
          false
        }).reserve(nme.id);
      }
    }
}


  pub fn player_xp_next(&self) -> i64 {
    self.player_level * 3
  }

  fn fill_region_ids(&mut self, position: Position, dir: Dir4) {
    let mut frontier: Vec<(Position, Dir4)> = vec!( (position, dir));
    while let Some((p,d)) = frontier.pop() {
      let rid = self.sealed.regions[p][d.index()];
      let t0 = self.sealed.board[p].contents[d.index()];

      let neighbors = subtile_neighbors((p,d));
      let mut min_rid = RegionId::MAX;

      for i in 0..4 { // find the greatest region id among matching neighbors
        let (np, nd) = neighbors[i];
        // the opposite is not considered adajcent if the center terrain doesn't match
        // UNLESS this is river terrain
        if i == 0
          && self.sealed.board[p].contents[4] != t0
          && t0 != Terrain::River {
            continue;
        }
        let t1 = self.sealed.board[np].contents[nd.index()];
        if t1 != t0 { continue; }
        let rid1 = self.sealed.regions[np][nd.index()];

        min_rid = min_rid.min(rid1);
      }

      for i in 0..4 { // walk matches with rid above min
        let (np, nd) = neighbors[i];
        // the opposite is not considered adajcent if the center terrain doesn't match
        if i == 0 && self.sealed.board[p].contents[4] != t0 { continue; }
        let t1 = self.sealed.board[np].contents[nd.index()];
        if t1 != t0 { continue; }
        let rid1 = self.sealed.regions[np][nd.index()];

        if min_rid < rid1 { frontier.push((np, nd)) }
      }

      if min_rid < rid {
        // if two compatible regions with distinct rids are adjacent,
        // the one with the larger id is merged into the smaller
        self.sealed.regions[p][d.index()] = min_rid;
        // we remove the larger from the rid start tracker
        if self.sealed.region_start.remove(&rid).is_some() {
          debug!("merged region {}", rid);
        }

        //if rid < RegionId::MAX {
        //  debug!("update cell regionid {} -> {}", rid, min_rid);
        //}
      }
    }
  }

  pub fn place_tile(&mut self, position: Position, tile: Tile) {
    self.sealed.board[position] = tile;
    { // region tracking
      // merge regions
      for d in Dir4::list() {
        self.fill_region_ids(position, d);
      }

      // new regions
      for d in Dir4::list() {
        self.fill_region_ids(position,d);
        if self.sealed.regions[position][d.index()] == RegionId::MAX {
          self.sealed.regions[position][d.index()] = self.sealed.next_region_id;
          self.sealed.region_start.insert(self.sealed.next_region_id, (position, d));
          self.sealed.next_region_id += 1;
        }
      }

      // update void frontier
      self.sealed.void_frontier.remove(position);
      for d in Dir4::list() {
        let n = position + d.into();
        if self.sealed.board[n] == Tile::default() {
          self.sealed.void_frontier.insert(n);
        }
      }

      // rebuild open regions
      self.sealed.open_regions.clear();
      for &void_cell in self.sealed.void_frontier.iter() {
        for d in Dir4::list() {
          let cell = void_cell + d.into();
          let regionid = self.sealed.regions[cell][d.opposite().index()];
          if regionid < RegionId::MAX {
            self.sealed.open_regions.insert(regionid);
          }
        }
      }

      // place quest
      if let Some(_) = self.sealed.next_quest {
        self.quests.insert(position, self.sealed.next_quest.take().unwrap());
      }
    }
  }

  pub fn update_region_sizes(&mut self) {
    self.sealed.region_sizes.clear();
    let mut v = vec![];
    for p in BOARD_RECT.iter() {
      v.clear();
      for d in Dir4::list() {
        let rid = self.sealed.regions[p][d.index()];
        // let terrain = self.sealed.board[p].contents[d.index()];
        if rid == RegionId::MAX { continue; }
        v.push(rid);
      }
      v.sort();
      v.dedup();
      // FIXME: algorithm is quadratic in region count
      // maybe replace linear map with hashmap
      for &rid in &v {
        *self.sealed.region_sizes.entry(rid).or_insert(0) += 1;
      }
    }
  }

  pub fn reward_completed_region(&mut self, rid: RegionId) {
    let (position, dir) = self.sealed.region_start[&rid];
    let terrain = self.sealed.board[position].contents[dir.index()];
    let size = self.sealed.region_sizes[&rid];
    if terrain == Terrain::River {
      // Cancel the reward if the region is a river without
      // a source
      let mut river_reward = false;
      let mut frontier: Vec<(Position, Dir4)> = vec![(position,dir)];
      let mut visited: Set<(Position, Dir4)>  = Set::new();
      while let Some(subtile@(tile,_)) = frontier.pop() {
        if self.sealed.board[tile].count(Terrain::River) == 1 {
          river_reward = true;
          break;
        }
        if !visited.contains(&subtile) {
          visited.insert(subtile);
          let neighbors = subtile_neighbors(subtile);
          for i in 0..4 {
            let n@(p,d) = neighbors[i];
            if i == 0 &&
              self.sealed.board[p].contents[4] != Terrain::River {
                continue;
            }
            if self.sealed.board[p]
              .contents[d.index()] == Terrain::River {
              frontier.push(n);
            }
          }
        }
      }
      if !river_reward { return; }
    }
    let xp_reward = if terrain == Terrain::Town {
      size.saturating_sub(1)
    } else {
      size.saturating_sub(REGION_REWARD_THRESHOLD)
    };
    let tile_reward = if size > REGION_REWARD_THRESHOLD { 1 } else { 0 };
    if xp_reward > 0 {
      let to = self.ui.layout[&HudItem::Xp].center();

      self.ui.animations.append_empty(0.).require(PLAYER_UNIT_ID);
      for _i in 0..3 {
        let delay = 0.15;
        self.defer_play_sound(xp_sound()).chain();
        self.ui.animations.append_empty(delay).chain();
      }

      for i in 0..xp_reward {
        let delay = i as f64 * 0.15;
        self.ui.animations.append_empty(0.).require(PLAYER_UNIT_ID);
        self.ui.animations.append_empty(delay).chain();
        self.launch_particle(self.player_pos, to, XP, YELLOW, 3., 0.03)
          .chain();
          self.add_xp(1).chain();
      }
    }
    if tile_reward > 0 {
      let to = self.ui.layout[&HudItem::Tile].center();
      self.ui.animations.append_empty(0.).require(PLAYER_UNIT_ID);
      self.defer_play_sound(tile_sound()).chain();
      self.launch_particle(self.player_pos, to, TILE, SKYBLUE, 3., 0.1).chain();
      self.add_tiles(1).chain();
    }
  }

  pub fn player_current_tile(&self) -> Tile {
    self.sealed.player_tile_transform * self.sealed.player_next_tile
  }

  // returns whether the next tile has any placeable spots
  pub fn next_tile(&mut self) -> bool {
    self.sealed.player_next_tile = tiles::generate(&mut self.rng);
    self.defer_set_hud(|hud| hud.tile_rotation = 0.)
      .reserve(PLAYER_UNIT_ID);
    self.add_tiles(-1).chain();

    // does the next tile have a quest?
    if roll_chance(&mut self.rng, QUEST_SPAWN_CHANCE) {
      debug!("quest");
      if let Some(quest) = eligible_for_quest(&self.enemies, &self.quests, &mut self.rng) {
        debug!("{:?}", quest);
        self.sealed.next_quest = Some(quest);
      }
    }

    for p in self.sealed.void_frontier.iter() {
      if self.tile_compatibility(*p, self.sealed.player_next_tile) > 0 {
        return true;
      }
    }
    false
  }

  pub fn update_player_dmap(&mut self) {
    self.sealed.player_dmap.fill(i16::MAX);
    let mut d = 0;
    let mut frontier = Vec::new();
    frontier.push(self.player_pos);
    let mut next_frontier = Vec::new();

    loop {
      while let Some(visit) = frontier.pop() {
        if self.sealed.player_dmap[visit] > d {
          self.sealed.player_dmap[visit] = d;
          for d in Dir4::list() {
            let neighbor = visit + d.into();
            if self.sealed.player_dmap[neighbor] == i16::MAX
              && self.sealed.board[neighbor] != Tile::default()
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
    self.sealed.nearest_enemy_dmap.fill(i16::MAX);
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
        if self.sealed.nearest_enemy_dmap[visit] > d {
          self.sealed.nearest_enemy_dmap[visit] = d;
          for d in Dir4::list() {
            let neighbor = visit + d.into();
            if self.sealed.nearest_enemy_dmap[neighbor] == i16::MAX
              && self.sealed.board[neighbor] != Tile::default()
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

        // animations
        let t0 = Vec2::from(from);
        let t1 = Vec2::from(to);
        let mid = (t0 + t1) / 2.;
        self.ui.animations.append(empty_animation)
          .reserve([from,to])
          .reserve(nme.id);
        self.animate_unit_motion(nme.id, t0, mid, 0.5 * BASE_ANIMATION_DURATION / speed)
          .chain();
        if self.sealed.board[from] == Tile::default() {
          let rgr = self.ragdoll_ref(nme.id).clone();
          self.ui.animations.append(move |_time| {
            unsafe {
              let ragdoll = rgr.get();
              ragdoll.img = enemy_img(nme.t, false);
              ragdoll.color = MONSTER_COLOR;
            }
            false
          }).chain();
        }
        self.animate_unit_motion(nme.id, mid, t1, 0.5 * BASE_ANIMATION_DURATION / speed)
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

  pub fn slay_enemy(&mut self, at: Position, dir: Dir4) {
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
    self.ui.animations.append_empty(0.).reserve(
      [id, PLAYER_UNIT_ID]
    ).reserve(at);
    self.defer_play_sound(xp_sound()).chain();
    self.animate_unit_fling(id, at.into(), velocity, 0.2)
      .require(id);
    self.add_hp(-1).require(id);
    self.ui.animations.append(empty_animation)
      .require([id, PLAYER_UNIT_ID]);
    self.launch_particle(
      at,
      self.ui.layout[&HudItem::Xp].center(),
      XP,
      YELLOW,
      3.,
      0.03,
    ).chain();
    self.add_xp(1).chain();
  }

  pub fn add_xp(&mut self, amount: i64) -> &mut Animation {
    self.player_xp += amount;
    let hud = self.ui.hud.clone();
    self.ui.animations.append(move |_| unsafe {
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
    if self.player_hp < self.score_min_hp {
      self.score_min_hp = self.player_hp;
    }
    let hudref = self.ui.hud.clone();
    let duration = 0.1;
    self.ui.animations.append(move |time| unsafe {
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
    self.defer_set_hud(move |hud| hud.turns += amount)
  }

  pub fn in_combat(&mut self) -> bool {
    let mut in_combat = false;
    for d in Dir4::list() {
      let adj = self.player_pos + d.into();
      // monsters in void don't count
      if self.sealed.board[adj] == Tile::default() { continue; }
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
      self.sealed.board[self.player_pos].contents[dir.index()];

    let second_half = Terrain::Road ==
      if self.sealed.board[target] == Tile::default() {
        self.player_current_tile().contents[opp.index()]
      } else {
        self.sealed.board[target].contents[opp.index()]
      };
    first_half && second_half
  }


  pub fn add_tiles(&mut self, amount: i64) -> &mut Animation {
    self.player_tiles += amount;
    self.defer_set_hud(move |hud| hud.tiles += amount)
  }

  fn ragdoll_ref(&mut self, unit_id: UnitId) -> Ref<Ragdoll> {
    if let Some(rgr) = self.ui.ragdolls.get(&unit_id) {
      (*rgr).clone()
    } else if unit_id == PLAYER_UNIT_ID {
      let rgr = Ref::new(Ragdoll {
        pos: self.player_relative_coordinates(Vec2::from(self.player_pos)),
        color: LIGHTGRAY,
        img: HERO,
        dead: false,
      });
      self.ui.ragdolls.insert(unit_id, rgr.clone());
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
      self.ui.ragdolls.insert(unit_id, rgr.clone());
      let result = rgr.clone();
      self.ui.animations.append(move |_| unsafe {
        rgr.get().color.a = 1.;
        false
      }).reserve(unit_id).reserve(pos);
      result
    }
  }

  pub fn animate_unit_fling(&mut self, u: UnitId, p0: Vec2, velocity: Vec2, duration: Seconds) -> &mut Animation {
    let uref = self.ragdoll_ref(u);
    self.ui.animations.append(move |time| {
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
    self.ui.animations.append(move |time| {
      let c = time.progress(duration);
        unsafe {
          uref.get().pos = c * prc1 + (1.-c) * prc0;
        }
      c < 1.
    })
  }

  pub fn launch_tile(
    &mut self,
    to: Position,
    tile: Tile,
    ) -> &mut Animation {
    let origin_pos = self.ui.layout[&HudItem::Tile].center();
    let p = Ref::new(AnimTile {
      pos: origin_pos,
      tile,
      dead: false
    });

    let cr = self.ui.camera_ref.clone();

    self.ui.flying_tiles.push(p.clone());

    let duration = 0.15;
    self.ui.animations.append(move |time: Time| {
      let c = time.progress(duration);
      let camera_focus = *cr;
      let target_board = Vec2::from(to - camera_focus);
      let target_screen_pos = DISPLAY_GRID.rect(target_board).center();
      unsafe{
        let it = p.get();
        it.pos = origin_pos * (1. - c) + target_screen_pos * c;
        it.dead = c >= 1.;
      }
      !p.dead
    })
  }

  pub fn launch_particle(
    &mut self,
    from: Position,
    to: ScreenCoords,
    img: Img,
    color: Color,
    kick: f64, // multiplier on initial (random) velocity
    decay: f64 // percentage of remaining distance remaining after a second
    ) -> &mut Animation {
    let p = Ref::new(Particle {
      pos: Vec2{x: f32::MAX, y: f32::MAX},
      img,
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

    self.ui.particles.push(p.clone());
    let origin = self.player_relative_coordinates(Vec2::from(from));
    let cr = self.ui.camera_ref.clone();

    self.ui.animations.append(move |time: Time| {
      if p.pos.x == f32::MAX {
        let camera_focus = Vec2::from(*cr);
        let origin_screen_pos = DISPLAY_GRID.rect(origin - camera_focus).center();
        unsafe { p.get().pos = origin_screen_pos; }
      }
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
    self.ui.hud.desire_path.push(to);

    self.animate_unit_motion(PLAYER_UNIT_ID, from.into(), to.into(), BASE_ANIMATION_DURATION.into())
      .reserve([from,to])
      .reserve(PLAYER_UNIT_ID);
    self.defer_set_hud(|hud|{
      hud.desire_path.remove(0);
    }).chain();

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
      let t2: Terrain = self.sealed.board[p2].contents[i2];
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
    self.ui.animations.tick();
    let mut died = vec!();
    for (id,v) in self.ui.ragdolls.iter() {
      if v.dead { died.push(*id); }
    }
    for dead in died {
      self.ui.ragdolls.remove(&dead);
    }
    for i in (0.. self.ui.particles.len()).rev() {
      if self.ui.particles[i].dead {
        self.ui.particles.remove(i);
      }
    }
    for i in (0..self.ui.flying_tiles.len()).rev() {
      if self.ui.flying_tiles[i].dead {
        self.ui.flying_tiles.remove(i);
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

  let mut resources = Resources::new(ASSETS);
  for path in LOAD_ME { resources.load_texture(path, FilterMode::Linear); }
  //load_sounds(&mut resources).await;

  let sounds: Map<Path, Rc<Sound>> = {
    let mut result = Map::new();
    for &path in SOUNDS_TO_LOAD {
      let file = ASSETS.get_file(path).expect(&format!("missing {}", path));
      let s: Sound = load_sound_from_bytes(file.contents())
        .await
        .expect(&format!("cant load {}", path));
        result.insert(path, Rc::new(s));
    }
    result
  };
  let volume = 0.3;
  let mut bgm = BGM::init(volume);


  let display_dim: Vec2 = DISPLAY_GRID.dim();
  let mut display = Display::new(resources, display_dim);

  let mut sim = SimulationState::new(&sounds);

  let mut debug_draw = false;

  loop {
    bgm.poll();

    if get_keys_pressed().len() > 0 {
      sim.ui.animations.hurry(2.);
    }

    let mut inputdir: Option<Dir4> = None;

    if let Some(input) = get_input() {
      if sim.ui.hud.defeat || sim.ui.hud.victory {
        sim = SimulationState::new(&sounds);
        next_frame().await;
        continue;
      }
      match input {
        Input::Mute => bgm.mute(),
        Input::Dir(dir) => {
          inputdir = Some(dir)
        }
        Input::Rotate1 => {
          sim.transform_tile(D8::R1);
        }
        Input::Rotate2 => {
          sim.transform_tile(D8::R3);
        }
        Input::Discard => {
          if sim.player_tiles > 0 {
            sim.sealed.next_quest = None;
            sim.next_tile();
          }
        }
        Input::LevelUp =>
          if sim.player_xp >= sim.player_xp_next() {
            sim.add_xp(-sim.player_xp_next());
            sim.player_hp_max += 1;
            let to = sim.ui.layout[&HudItem::Hp].center();
            sim.defer_play_sound(LEVEL_UP_SOUND);
            sim.launch_particle(sim.player_pos, to,
              HEART, RED,
              3., 0.02
            ).chain();
            sim.full_heal().chain();
            sim.player_level += 1;
          }
      }
    }

    let mut tile_placed: bool = false;
    let mut player_moved: bool = false;
    let mut needs_road = false;
    let mut can_move = true;
    let mut tile_compat: bool = true;

    if let Some(playermove) = inputdir  {
      let target = sim.player_pos + playermove.into();
      let target_empty = sim.sealed.board[target] == Tile::default();

      // do combat
      if sim.in_combat() {
        let mut defeated_boss = false;
        if let Some(Enemy { t: EnemyType::GhostWitch, .. }) = sim.enemies.get(target) {
          let mut speed_mul: f64 = 1.;
          while sim.num_bosses > 1 {
            let id = sim.enemies.get(target).unwrap().id;
            let delay = BASE_ANIMATION_DURATION/speed_mul;
            sim.ui.animations.append_empty(delay)
              .reserve(id);
            sim.slay_enemy(target, playermove);
            sim.num_bosses -= 1;
            sim.defer_set_hud(|hud| hud.bosses -= 1).reserve(id);
            sim.spawn_enemy(EnemyType::GhostWitch, target);
            sim.set_enemy_alerts(true);
            speed_mul += 0.5;
            if sim.player_dead() { break; }
          }
          if !sim.player_dead() {
            defeated_boss = true;
          }
        }
        let crowd: Map<Position, u8> = sim.calculate_crowd(target);
        if crowd.len() > 0 { // fight!
          player_moved = true;
          let mut speed_mul: f64 = 1.;
          while sim.enemies.contains_key(target) {
            if sim.player_dead() { break; }
            speed_mul += 0.5;
            sim.slay_enemy(target, playermove);
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
          }

          let won = defeated_boss && !sim.player_dead();
          sim.ui.animations.sync();
          sim.defer_set_hud(move |hud| hud.victory = won).chain();
        } else { // nobody in this spot to fight
          needs_road = true;
        }
      }


      let using_road = sim.is_road_dir(playermove);
      can_move = can_move && (!needs_road || using_road);
      can_move = can_move && (!target_empty || sim.tile_compatibility(target, sim.player_current_tile()) > 0);
      if sim.player_tiles < 1  && sim.sealed.board[target] == Tile::default() {
        can_move = false;
      }
      if !player_moved && can_move { // move player

        let target_is_slow: bool = {
          let mut rivers = 0;
          let t = sim.sealed.board[target];
          for i in 0..4 {
            if t.contents[i] == Terrain::River { rivers += 1; }
          }
          rivers >= 2
        };
        let edge_is_slow: bool = {
          let t0 = sim.sealed.board[sim.player_pos]
            .contents[playermove.index()];
          let t1 = sim.sealed.board[target]
            .contents[playermove.opposite().index()];
          t0 == Terrain::River
            && t1 == Terrain::River
        };

        // try to place tile
        if sim.sealed.board[target] == Tile::default() && sim.player_tiles > 0 {
          sim.place_tile(target, sim.player_current_tile());
          sim.sealed.score_tiles_placed += 1;
          unsafe {
            sim.ui.hud.get().hidden_spaces.insert(target);
          }
          sim.ui.animations.append_empty(0.).reserve(target).reserve(PLAYER_UNIT_ID);
          sim.launch_tile(target, sim.player_current_tile()).chain();
          sim.defer_set_hud(move |hud|{ hud.hidden_spaces.remove(target);} )
            .chain();
          sim.defer_play_sound(PLACE_TILE_SOUND).chain();
          tile_compat = sim.next_tile();
          tile_placed = true;
          // new tiles smoosh monsters
          if let Some(nme) = sim.enemies.remove(target) {
            sim.ui.ragdolls.remove(&nme.id);
          }
          sim.update_region_sizes();

          { // check for perfect tile bonuses
            // on placed tile and neighbors
            let mut to_check = vec!(target);
            for d in Dir4::list() {
              to_check.push(target + d.into());
            }
            for &p in &to_check {
              let mut is_matched = true;
              let ptile = sim.sealed.board[p];

              for d in Dir4::list() {
                let ntile = sim.sealed.board[p + d.into()];
                if ptile.contents[d.index()]
                  != ntile.contents[d.opposite().index()] {
                    is_matched = false;
                }
              }
              if is_matched {
                let to = sim.ui.layout[&HudItem::Tile].center();
                sim.ui.animations.append_empty(0.).require(PLAYER_UNIT_ID);

                sim.defer_set_hud(move |hud| {
                  hud.highlighted_spaces.insert(p);
                });
                sim.defer_play_sound(tile_sound()).chain();
                sim.launch_particle(p, to, TILE, SKYBLUE, 0.5, 0.1).chain();
                sim.add_tiles(1).chain();
                sim.defer_set_hud(move |hud| {
                  hud.highlighted_spaces.remove(p);
                }).chain();


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
            let p2 = target + d.into();
            let d2 = d.opposite();
            for regionid in [
              sim.sealed.regions[target][d.index()],
              sim.sealed.regions[p2][d2.index()]
            ] {
              if regionid == RegionId::MAX { continue; }
              if !sim.sealed.open_regions.contains(&regionid) {
                just_completed.insert(regionid);
              }
            }
          }
          for &regionid in just_completed.iter() {
            sim.reward_completed_region(regionid);
          }
        } else { // we stepped on an existing tile
          if (target_is_slow || edge_is_slow) && !using_road {
            let to = sim.ui.layout[&HudItem::SpeedPenalty].center();
            sim.ui.animations.append(empty_animation).require(target);
            sim.launch_particle(target, to,
              TIME, BLUE, 0.4, 0.03
            ).chain();
            sim.add_monster_turns(1).chain();
          }
        }



        // clear monster alerts
        sim.set_enemy_alerts(false);
        sim.move_player(target);
        player_moved = true;
        //debug!("player: {:?}", sim.player_pos);

        { // Quest reward, spawn quest items
          let mut fulfilled_quests: WrapMap<Quest> = WrapMap::new(BOARD_RECT);
          for (&p, &q) in sim.quests.clone().iter() {
            if q.quota < 1 && sim.player_hp > 0 {
              let distance = torus_max_norm(BOARD_RECT, p - target);
              if distance >= 4 { continue; }

              fulfilled_quests.insert(p, q);
              sim.quests.remove(p);
              sim.prizes.insert(p, Prize::Heal);
              let to = sim.ui.layout[&HudItem::Tile].center();
              for i in 0..(QUEST_REWARD as u8) {
                let delay = f64::from(i)* 0.7 * BASE_ANIMATION_DURATION ;
                sim.ui.animations.append_empty(0.).require(PLAYER_UNIT_ID);
                sim.ui.animations.append_empty(delay).chain();
                sim.defer_play_sound(tile_sound()).chain();
                sim.launch_particle(p, to, TILE, SKYBLUE, 3., 0.1).chain();
                sim.add_tiles(1).chain();
              }
            }
          }
          for p in fulfilled_quests.keys() {
            sim.quests.remove(*p);
          }
        }


        // try to collect prize
        if let Some(&prize) = sim.prizes.get(target) {
          sim.prizes.remove(target);
          let to = sim.ui.layout[&HudItem::Hp].center();
          sim.ui.animations.append_empty(0.).reserve(PLAYER_UNIT_ID);
          sim.defer_play_sound(LEVEL_UP_SOUND).chain();
          sim.launch_particle(target, to,
            prize_img(prize), RED,
            3., 0.02
          ).chain();
          sim.full_heal().chain();
        }
      }


      if sim.player_dead() {
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
        sim.defer_set_hud(|hud| hud.defeat = true).chain();
      }
      if !player_moved { sim.ui.compass_flash = 0.6; }
    }

    //debug!("{:?}", sim.player_pos);
    let camera_offset: IVec = display.camera_focus - sim.player_pos;
    display.camera_focus = sim.player_pos + CAMERA_TETHER.clamp_pos(camera_offset);
    unsafe {
      *sim.ui.camera_ref.get() = display.camera_focus;
    }

    {//monsters
      let mut monsters_go = false;
      if tile_placed || (player_moved && sim.player_tiles < 1) {
        sim.ui.animations.sync_positions();
        sim.add_monster_turns(1).chain();
        monsters_go = true;
        sim.update_player_dmap();
      }
      let mut spawns = vec!();

      let mut acceleration = 1.0;
      while monsters_go && sim.monster_turns > 0 {
        sim.sealed.enemy_supply += sim.sealed.void_frontier.len() as i64;
        spawns.clear();
        sim.update_nearest_dmap();
        //do monster turn
        for (&pos, &_nme) in sim.enemies.clone().iter() {
          let maybe_pos = enemy_pathfind(&mut sim, pos);
          if let Some(new_pos) = maybe_pos {
            sim.move_enemy(pos, new_pos, acceleration);
          }
          //debug!("a monster turn happened at {:?}", pos)
        }
        //spawn monsters maybe
        for &p in sim.sealed.void_frontier.iter() {
          if sim.enemies.contains_key(p) {
            // don't spawn a monster if there's already a monster
            continue;
          }
          if ((sim.rng.next_u64() % 5000) as i64 ) < sim.sealed.enemy_supply {
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

        sim.ui.animations.append_empty(BASE_ANIMATION_DURATION / acceleration).chain();
        sim.ui.animations.sync_positions().chain();
        sim.add_monster_turns(-1).chain();
        acceleration += 0.5;
      }

      sim.set_enemy_alerts(true);

    }

    sim.tick_animations();


    let scale: f32 = f32::min(
      screen_width() / display.dim.x as f32,
      screen_height() / display.dim.y as f32,
    );

    const DRAW_BOUNDS:IRect = IRect{ x: -9, y:-8, width: 18, height: 17};
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
        if sim.sealed.board[p] == Tile::default() { continue; }
        let r = display.pos_rect(p.into());
        draw_rectangle(r.x, r.y, r.w, r.h, DARKBROWN);
        //display.draw_tile_1(r, tile, terrain);
      }
      // draw terrain
      for &terrain in Terrain::DRAW_ORDER {
        for offset in DRAW_BOUNDS.iter() {
          let p = sim.player_pos + offset;
          let mut tile = sim.sealed.board[p];
          if terrain == Terrain::None && sim.ui.hud.hidden_spaces.contains(p) {
            tile = Tile::default();
          }
          let r = display.pos_rect(p.into());
          display.draw_tile_1(r, tile, terrain, 0.);

        }
      }
      // draw region hints
      for rid in &sim.sealed.open_regions {
        let font_size = 40;
        if let Some(sz) = sim.sealed.region_sizes.get(&rid) {
          if *sz > REGION_REWARD_THRESHOLD {
            let (pos, dir) = sim.sealed.region_start.get(&rid).unwrap();
            let terrain = sim.sealed.board[*pos].contents[dir.index()];
            let mut xp = *sz - REGION_REWARD_THRESHOLD;
            if terrain == Terrain::Town {
              xp = *sz - 1
            };
            let mut r = display.pos_rect((*pos).into());
            let mut offset = 64 * IVec::from(*dir);
            offset.y *= -1; // screen vs map coordinate shenanigans
            r = r.offset(offset.into());
            display.draw_img(r, terrain.color(), &FLAG);

            let text = format!("{}", xp);
            draw_text(&text, r.center().x + 20., r.center().y - 18., font_size as f32, BLACK);
          }
        }
      }
      // draw terrain highlights
      for offset in DRAW_BOUNDS.iter() {
        let p = sim.player_pos + offset;
        let r = display.pos_rect(p.into());
        if sim.ui.hud.highlighted_spaces.contains(p) {
          display.draw_img(r, SKYBLUE, &BOX);
        }
      }
      for offset in DRAW_BOUNDS.iter() { // draw quests and prizes
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

      if debug_draw {
        debug!("----------------");
      }
      // tile placement hints
      for offset in DRAW_BOUNDS.iter() {
        let p = sim.player_pos + offset;
        if !sim.sealed.void_frontier.contains(p) { continue; }
        {
          let compat = sim.tile_compatibility(p, sim.player_current_tile());
          if compat == 1 {
            display.draw_grid( p.into(), DARKGRAY, &BOX);
          } else if compat == 2 {
            display.draw_grid( p.into(), SKYBLUE, &BOX);
          }
        }

        // rotation hints
        let mut g = D8::E;
        for _ in 0..4 {
          let rt = g * sim.sealed.player_next_tile;
          let compat = sim.tile_compatibility(p, rt);
          if debug_draw {
            debug!("pos {:?} compat {} @ {:?}", p, compat, g);
          }
          if compat > 0 {
            let color = if compat == 1 { DARKGRAY } else { SKYBLUE };
            let r = (g * Dir4::Right).radians();
            display.draw_grid_r(p.into(), color, &HINT, r);
          }
          g = g * D8::R1;
        }
      }
      if debug_draw {
        debug!("----------------");
      }
      debug_draw = false;

      // draw enemies
      for ragdoll in sim.ui.ragdolls.values() {
        display.draw_grid(
          ragdoll.pos,
          ragdoll.color,
          &ragdoll.img
        );
      }

      // draw player path
      {
        let n = sim.ui.hud.desire_path.len();
        if n > 1 {
          for i in 1..n {
            let prev = sim.ui.hud.desire_path[i-1];
            let here = sim.ui.hud.desire_path[i];
            let Ok(dir) = Dir4::try_from(here - prev) else { continue };
            let is_end = i == (n-1);
            let r = display.pos_rect(here.into());
            display.draw_img(r,
              YELLOW,
              &path_img(dir, is_end)
            );
            if is_end { continue; }
            let next = sim.ui.hud.desire_path[i+1];
            let Ok(nextdir) = Dir4::try_from(here - next) else { continue };
            display.draw_img(r,
              YELLOW,
              &path_img(nextdir, is_end)
            );

          }
        }
      }

      // draw boss count
      for offset in DRAW_BOUNDS.iter() { // draw quests and prized
        let p = sim.player_pos + offset;
        if p != BOSS_LOCATION { continue; }
        let r = display.pos_rect(p.into());
        let text = format!("{}", sim.ui.hud.bosses);
        let font_size = 70;
        if sim.ui.hud.bosses >= 2 {
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
        let sz = DISPLAY_GRID.tile_size;       // without tile margin

        { // Bar
          let x = 0.;
          let h = sz.y + 2. * margin;
          let y = display.dim.y - h;
          let w = display.dim.x;
          let rect = Rect { x, y, w, h };
          draw_rectangle(x, y, w, h, DARKGRAY);
          sim.ui.layout.insert(HudItem::Bar, rect);
        }

        if sim.ui.hud.defeat {
            let bar = sim.ui.layout[&HudItem::Bar];
            let display_text = format!("Defeated...");
            let textdim: TextDimensions = measure_text(&display_text, None, font_size, font_scale);
            let leftover = bar.h - textdim.height;
            let x = (display.dim.x - textdim.width - margin)/2.;
            let y = bar.y + (0.5 * leftover) + textdim.offset_y;
            draw_text(&display_text, x, y, font_size as f32, WHITE);
        } else {
          { // Next Tile
            let hudbar: Rect = sim.ui.layout[&HudItem::Bar];
            let r = Rect {
              x: hudbar.w - sz.x - margin,
              y: hudbar.y + margin ,
              w: sz.x,
              h: sz.y
            };
            sim.ui.layout.insert(HudItem::Tile, r);
            if sim.player_tiles > 0 {
              display.draw_tile(
                r,
                sim.ui.hud.tile_transform * sim.sealed.player_next_tile,
                sim.ui.hud.tile_rotation as f32
              );
              display.draw_img_r(
                r,
                WHITE,
                &HINT,
                (sim.ui.hud.tile_transform * Dir4::Right).radians() + sim.ui.hud.tile_rotation,
              );
            }
            if let Some(q) = sim.sealed.next_quest {
              draw_quest(&display, &r, &q);
            }
          }

          { // Remaining tiles
            let r = sim.ui.layout[&HudItem::Tile];
            let bar = sim.ui.layout[&HudItem::Bar];
            let remaining_tiles = format!("{}", sim.ui.hud.tiles);
            let textdim: TextDimensions = measure_text(&remaining_tiles, None, font_size, font_scale);
            let leftover = bar.h - textdim.height;
            let x = r.x - textdim.width - margin;
            let y = bar.y + (0.5 * leftover) + textdim.offset_y;
            draw_text(&remaining_tiles, x, y, font_size as f32, WHITE);
          }

          { // movement arrows
            let bar = sim.ui.layout[&HudItem::Bar];
            let rect = Rect {
              x: bar.x + margin,
              y: bar.y + margin,
              w: sz.x,
              h: sz.y,
            };

            const BLINK: f32 = 0.1;
            let arrow_color = if sim.ui.compass_flash > 0. &&
              (sim.ui.compass_flash % (2. * BLINK) > BLINK) {
              RED
            } else {
              WHITE
            };

            sim.ui.layout.insert(HudItem::Arrows, rect);
            for d in Dir4::list() {
              let target = sim.player_pos + d.into();
              if sim.sealed.void_frontier.contains(target) {
                let compat = sim.tile_compatibility(target, sim.player_current_tile());
                // tile doesnt fit
                if compat == 0 { continue; }
                // no tiles left
                if sim.player_tiles < 1 { continue; }
              }
              if sim.in_combat() && !sim.is_road_dir(d) {
                // we can't step on void
                if sim.sealed.board[target] == Tile::default() { continue; }
                // we can't step on a free space
                if !sim.enemies.contains_key(target) { continue; }
              }
              display.draw_img(
                rect,
                arrow_color,
                &arrow_img(d),
              )
            }
          }

          { // Current/Max HP and XP
            let bar = sim.ui.layout[&HudItem::Bar];
            let arrows = sim.ui.layout[&HudItem::Arrows];
            let blink = get_time() % (2. * BLINK) > BLINK;

            let hp = format!("HP: {}/{} ", sim.ui.hud.hp, sim.player_hp_max);
            let hpdim: TextDimensions = measure_text(&hp, None, font_size, font_scale);
            let xp = format!("XP: {}/{}", sim.ui.hud.xp, sim.player_xp_next());
            let xpdim: TextDimensions = measure_text(&xp, None, font_size, font_scale);
            let leftover = bar.h - hpdim.height - xpdim.height;
            let x = margin + arrows.x + arrows.w;
            let hpr = Rect {
              x,
              w: hpdim.width,
              h: hpdim.height,
              y: bar.y + (0.33 * leftover),
            };
            let xpr = Rect {
              x,
              w: xpdim.width,
              h: xpdim.height,
              y: bar.y + (0.66 * leftover) + hpr.h,
            };
            let hp_color = if sim.player_hp == 1 && blink { RED } else { sim.ui.hud.hp_color };
            draw_text(&hp, hpr.x, hpr.y + hpdim.offset_y, font_size as f32, hp_color);
            const BLINK: f64 = 1.;
            let mut xp_color = WHITE;
            let can_level = sim.ui.hud.xp >= sim.player_xp_next();
            if can_level && blink {
              xp_color = YELLOW;
            };
            draw_text(&xp, xpr.x, xpr.y + xpdim.offset_y, font_size as f32, xp_color);
            sim.ui.layout.insert(HudItem::Hp, hpr);
            sim.ui.layout.insert(HudItem::Xp, xpr);
          }


          { // num monster turns
            let bar = sim.ui.layout[&HudItem::Bar];
            let icon_rect = Rect{
              x: bar.w * 0.5,
              y: bar.y + margin,
              w: sz.x,
              h: sz.y,
            };
            if sim.ui.hud.turns > 0 {
              let text = format!("{}", sim.ui.hud.turns);
              let textdim = measure_text(&text, None, font_size, font_scale);
              let y = bar.y + 0.5 * (bar.h - textdim.height) + textdim.offset_y;
              let x = icon_rect.x - textdim.width - margin;
              display.draw_img( icon_rect, BLUE, &TIME);
              draw_text(&text,x,y, font_size.into(), WHITE);
            }
            sim.ui.layout.insert(HudItem::SpeedPenalty, icon_rect);
          }

          if !tile_compat { // discard hint
            let bar = sim.ui.layout[&HudItem::Bar];
            let tile = sim.ui.layout[&HudItem::Tile];
            let hint = "[X] to discard";
            let hint_dim: TextDimensions = measure_text(hint, None, font_size, font_scale);
            let hint_rect = Rect {
              x: tile.x + tile.w - hint_dim.width,
              y: bar.y - hint_dim.height - margin,
              h: hint_dim.height,
              w: hint_dim.width,
            };
            draw_text(hint, hint_rect.x, hint_rect.y, font_size as f32, sim.ui.hud.hint_color);
            sim.ui.layout.insert(HudItem::DiscardHint, hint_rect);
          }

          // level up hint
          if sim.ui.hud.xp >= sim.player_xp_next() && !sim.player_dead() {
            let xp = sim.ui.layout[&HudItem::Xp];
            let hint = "[Z]";
            let hint_dim: TextDimensions = measure_text(hint, None, font_size, font_scale);
            let hint_rect = Rect {
              x: xp.x + xp.w + margin,
              y: xp.y + hint_dim.offset_y,
              h: hint_dim.height,
              w: hint_dim.width,
            };
            draw_text(hint, hint_rect.x, hint_rect.y, font_size as f32, sim.ui.hud.hint_color);
            sim.ui.layout.insert(HudItem::LevelHint, hint_rect);
          }
        }
      }

      { // draw dmap2
        // let dmap = &sim.sealed.nearest_enemy_dmap;
        // for offset in (IRect{ x: -8, y:-8, width: 17, height: 17}).iter() {
        //  let p = sim.player_pos + offset;
        //  let dmapvalue = dmap[p];
        //  if dmapvalue > 20 {
        //    continue;
        //  }
        //  //let tile = sim.sealed.board[p];
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
        for p in &sim.ui.particles {
          let r = Rect{x:-32., y: -32., w: 64., h: 64.}.offset(p.pos);
          display.draw_img(r, p.color, &p.img);
        }
        for t in &sim.ui.flying_tiles {
          let r = Rect{x:-64., y: -64., w: 128., h: 128.}.offset(t.pos);
          display.draw_tile(r, t.tile, 0.);
        }
      }
    }

    if sim.ui.hud.victory {
      clear_background(BLACK);
      let score = sim.score_min_hp * sim.sealed.score_tiles_placed;
      let mut y = 300.;
      let margin = 15.;
      let font_size = 64;
      let color = WHITE;
      let mut i = 0;
      for text in &[
        "Victory!",
        &format!("Tiles Placed {} ", sim.sealed.score_tiles_placed),
        &format!("Minimum HP {}", sim.score_min_hp),
        &format!("Final Score {}", score),
      ] {
        let metrics = measure_text(text, None, font_size, 1.);
        let x = 0.5 * (display.dim.x - metrics.width);
        draw_text(text, x, y, font_size as f32, color);
        if i == 0 {
          y += margin * 2.;
          i += 1;
        }
        y += metrics.height + margin;
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

    sim.ui.compass_flash -= get_frame_time();
    decay_sounds(get_frame_time());
    next_frame().await;

  }

}

fn select_candidate(mut candidates: Vec<Position>, sim: &mut SimulationState) -> Option<Position> {
  // filter out invalid tiles
  let mut valid: Vec<IVec> = Vec::new();
  for c in candidates.drain(0..) {
    if sim.sealed.board[c] != Tile::default() && !sim.enemies.contains_key(c) {
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
  let mut valid: Vec<Dir4> = forest_edges(&pos, &sim.sealed.board);
  debug!("forest dirs {:?} for {:?}", valid, sim.enemies[pos]);
  if valid.len() == 0 {
    // no forest edges means anything is a candidate
    valid = Dir4::list().into();
  }
  else {
    // still have a chance to escape the forest
    for d in Dir4::list().iter() {
      if roll_chance(&mut sim.rng, FOREST_ESCAPE_CHANCE) {
        valid.push(*d);
      }
    }
  }
  debug!("valid dirs {:?} for {:?}", valid, sim.enemies[pos]);

  let mut candidates: Vec<IVec> = Vec::new();
  for &d in &valid {
    let target = pos + IVec::from(d);
    // dont step on me
    if equivalent(target, sim.player_pos) { continue; }
    // no void
    if sim.sealed.board[target] == Tile::default() { continue; }
    // dont step on quest
    if sim.quests.contains_key(target) { continue; }
    // dont step on prize
    if sim.prizes.contains_key(target) { continue; }
    candidates.push(target);
  }
  if sim.sealed.board[pos] != Tile::default() {
    candidates.push(pos);
  }
  match sim.enemies[pos].t {
    EnemyType::Clyde => {}
    EnemyType::Blinky => {
      let mut min_score: i16 = i16::MAX;
      for &c in &candidates {
        min_score = min_score.min(sim.sealed.player_dmap[c]);
      }
      candidates = candidates.drain(..).filter(|c|{
        min_score == sim.sealed.player_dmap[*c]
      }).collect();
    }
    EnemyType::Pinky => {
      let mut max_score: i16 = i16::MIN;
      for &c in &candidates {
        max_score = max_score.max(sim.sealed.nearest_enemy_dmap[c]);
      }
      candidates = candidates.drain(..).filter(|c|{
        max_score == sim.sealed.nearest_enemy_dmap[*c]
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
    let quota = nme_counts.get(selected_type).unwrap().max(&QUEST_MIN);
    quest.target = *selected_type;
    quest.quota = *quota;
    Some(quest)
  } else {
    None
  }
}
