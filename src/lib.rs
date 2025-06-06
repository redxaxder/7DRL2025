
pub type Seconds = f64;
pub type UnitId = u64;
pub type DMap = Buffer2D<i16>;

pub const BOARD_RECT: IRect = IRect { x: 0, y:0, width: 50, height: 50 };
pub const PLAYER_UNIT_ID: UnitId = 0;

pub const MONSTER_COLOR: Color = WHITE; // Color{r: 0.88, g:0.28, b: 0.7, a: 1.};

pub const INVISIBLE: Color = Color{r:0.,g:0.,b:0.,a:0.};

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

pub mod tiles;

pub mod resources;
pub use resources::*;

pub mod util;
pub use util::*;

pub mod fov;


pub use macroquad::prelude::*;


pub mod footguns;


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
}
impl Terrain {
  pub const DRAW_ORDER: &[Self] = &[
  Self::River,
  Self::Road,
  Self::Grass,
  Self::Town,
  Self::Forest,
  Self::None,
  ];

  pub fn index(self) -> usize {
    unsafe {
      std::mem::transmute::<Self, u8>(self) as usize
    }
  }

  pub fn color(self) -> Color {
    TERRAIN_COLOR[self.index()]
  }

  pub fn requires_match(self) -> bool {
    match self {
      Self::River => true,
      Self::Road => true,
      _ => false,
    }
  }

  pub fn draw16(self) -> bool {
    match self {
      Self::Road => true,
      Self::Town => true,
      Self::River => true,
      _ => false
    }
  }
}


const TERRAIN_COLOR: &[Color] = &[
  BLACK,
  Color{r:0., g:0.6, b:0.2, a: 1.},
  GOLD,
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

impl Tile {
  pub fn count(self, t: Terrain) -> usize {
    let mut n = 0;
    for i in 0..4 {
      if self.contents[i] == t { n += 1; }
    }
    n
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
    let mut contents = rhs.contents.clone();
    for d in Dir4::list() {
      let d2 = self * d;
      contents[d2.index()] = rhs.contents[d.index()];
    }
    Tile {contents}
  }
}


#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub enum EnemyType {
  Clyde, //moves randomly
  Blinky, //chases player
  Pinky, //avoids other enemies
  GhostWitch, //the boss
}

impl EnemyType {
  pub const fn list() -> [EnemyType; 4] {
    const LIST: [EnemyType;4] = [EnemyType::Clyde,
                                 EnemyType::Blinky,
                                 EnemyType::Pinky,
                                 EnemyType::GhostWitch];
    LIST
  }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub enum Prize {
  Heal,
}

static mut NEXT_UNIT_ID: UnitId = 10;
fn next_unit_id() -> UnitId {
  unsafe {
    let r = NEXT_UNIT_ID;
    NEXT_UNIT_ID += 1;
    r
  }
}


#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Enemy {
  pub id: UnitId,
  pub t: EnemyType
}

impl Enemy {
  pub fn new(nme_type: EnemyType) -> Self {
    let id = next_unit_id();
    let t = nme_type;
    Enemy { id, t}
  }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Quest {
  pub target: EnemyType,
  pub quota: u64,
  pub id: u64,
}

impl Quest {
  pub fn new() -> Self {
    let target = EnemyType::Blinky;
    let quota = 0;
    let id = next_unit_id();

    Quest {target, quota, id}
  }
}


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

  pub fn get(&self, k: IVec) -> Option<&V> {
    self.map.get(
      &self.rect.wrap(k)
    )
  }

  pub fn remove(&mut self, k: IVec) -> Option<V> {
    self.map.remove(
      &self.rect.wrap(k)
    )
  }
  pub fn contains_key(&self, k: IVec) -> bool {
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

  pub fn entry(&mut self, k: IVec) -> Entry<IVec, V> {
    self.map.entry(k)
  }

  pub fn get_mut(&mut self, k: IVec) -> Option<&mut V> {
    self.map.get_mut(&k)
  }

  pub fn iter_mut(&mut self) -> IterMut<IVec, V> {
    self.map.iter_mut()
  }

}

#[derive(Clone, Debug)]
pub struct WrapSet {
  rect: IRect,
  set: Set<Position>
}
impl WrapSet {
  pub fn new(rect: IRect) -> Self {
    Self { rect, set: Set::new() }
  }

  pub fn len(&self) -> usize {
    self.set.len()
  }

  pub fn insert(&mut self, k: IVec) {
    self.set.insert(self.rect.wrap(k));
  }

  pub fn remove(&mut self, k: IVec) -> bool {
    self.set.remove(
      &self.rect.wrap(k)
    )
  }
  pub fn contains(&self, k: IVec) -> bool {
    self.set.contains(
      &self.rect.wrap(k)
    )
  }

  pub fn iter(&self) -> set::Iter<IVec> {
    self.set.iter()
  }

  pub fn clear(&mut self) {
    self.set.clear()
  }
}


impl<V> std::ops::Index<IVec> for WrapMap<V> {
  type Output = V;

  fn index(&self, index: IVec) -> &Self::Output {
    &self.map[&self.rect.wrap(index)]
  }
}

pub fn roll_chance(rng: &mut Rng, chance: u64) -> bool {
  rng.next_u64() % 1000 < chance
}

pub fn wrap_rect(rect: Rect, v: Vec2) -> Vec2 {
  Vec2 {
    x: wrap1f(v.x, rect.x, rect.w),
    y: wrap1f(v.y, rect.y, rect.h),
  }
}

fn wrap1f(x: f32, min: f32, width: f32)  -> f32 {
  (x - min).rem_euclid(width) + min
}


pub fn equivalent(p:Position, q:Position) -> bool {
  BOARD_RECT.wrap(p) == BOARD_RECT.wrap(q)
}


pub fn subtile_neighbors(st: (Position, Dir4)) -> [(Position, Dir4);4] {
  let (p,d) = st;
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
  neighbors
}

pub fn draw_quest(display: &Display, r: &Rect, quest: &Quest) {
  display.draw_img(*r, BEIGE, &SCROLL);
  let mut er: Rect = r.clone();
  er.scale(0.5, 0.5);
  er = er.offset(Vec2{ x: r.w/4., y: r.h /4. });
  display.draw_img(er, BLACK, &enemy_img(quest.target, false));
  let quest_text = format!("{}", quest.quota);
  let font_size = 50;
  let font_scale = 1.;
  let textdim: TextDimensions = measure_text(&quest_text, None, font_size, font_scale);
  let margin = 0.;
  let text_x = er.x + textdim.width * 0.8 - margin;
  let text_y = er.y + textdim.height * 0.2;
  draw_text(&quest_text, text_x, text_y, font_size.into(), BLACK);
}


pub fn torus_max_norm(bounds: IRect, v: IVec) -> i16 {
  let w = bounds.wrap(v);
  let y = (w.y - bounds.y).min(bounds.y + bounds.height - w.y);
  let x = (w.x - bounds.x).min(bounds.x + bounds.width - w.x);
  x.max(y)
}


use macroquad::audio::Sound;
use std::rc::Rc;


const MAX_VOLUME: f32 = 2.0;
// assumed rate at which sounds stop playing
// lower value = allow more new sounds to play loud sooner
const VOLUME_DECAY: f32 = 0.05;
struct SoundQueue{
  volume: f32,

}
impl SoundQueue {
  const fn new() -> Self {
    Self { volume: 0. }
  }

  fn decay(&mut self, time: f32) {
    self.volume *= VOLUME_DECAY.powf(time);
  }
}


static mut SOUND_QUEUE: SoundQueue = SoundQueue::new();

pub fn decay_sounds(time: f32) {
  unsafe {
    #[allow(static_mut_refs)]
    SOUND_QUEUE.decay(time)
  }
}

pub fn play_sound(sound: Rc<Sound>) {
  unsafe {
    let playing_volume = SOUND_QUEUE.volume;
    let remaining_volume = MAX_VOLUME - playing_volume;
    let volume = 1.0_f32.min(remaining_volume * 0.5);
    SOUND_QUEUE.volume += volume;
    macroquad::audio::play_sound(
      &sound,
      macroquad::audio::PlaySoundParams {
        looped: false,
        volume,
      }
    );
  }
}


struct Noop;
impl std::task::Wake for Noop {
    fn wake(self: std::sync::Arc<Self>) {}
}

use std::pin::Pin;
use std::future::Future;
pub struct BGM {
  future: Pin<Box<dyn Future<Output = Option<Sound>>>>,
  done: bool,
  mute: bool,
  sound: Option<Sound>,
  volume: f32,
}
impl BGM {
  async fn load_bgm(volume: f32) -> Option<Sound> {
    let path = "bgm/anewdayshurry.wav";
    let sound = macroquad::audio::load_sound(&path).await.ok()?;
    macroquad::audio::play_sound(
      &sound,
      macroquad::audio::PlaySoundParams {
        looped: true,
        volume,
      }
    );
    Some(sound)
  }

  pub fn init(volume: f32) -> Self {
    let future = Box::pin(Self::load_bgm(volume));
    let done = false;
    Self {future, done, volume, mute: false, sound: None}
  }

  pub fn mute(&mut self) {
    let Some(ref sound) = self.sound else { return; };
    self.mute = !self.mute;
    let v = if self.mute {0.} else {self.volume};
    macroquad::audio::set_sound_volume(&sound, v);
  }

  pub fn poll(&mut self) {
    if self.done { return; }

    let p = Pin::as_mut(&mut self.future);
    let waker: std::task::Waker = std::sync::Arc::new(Noop).into();
    let mut ctx = std::task::Context::from_waker(&waker);
    match Future::poll(p, &mut ctx) {
        std::task::Poll::Ready(sound) => {
          self.sound = sound;
          self.done = true
        }
        _ => {}
    }
  }
}

