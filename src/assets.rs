use crate::*;
use include_dir::{Dir, include_dir};

pub const ASSETS: Dir<'static> =  include_dir!("$CARGO_MANIFEST_DIR/assets");

pub const LOAD_ME: &[&'static str] = &[
  "hero.png",
  "terrain_placeholder.png",
  "terrain_grass.png",
  "terrain_forest.png",
  "npc.png",
  "nme1.png",
  "nme2.png",
  "nme3.png",
  "nme4.png",
  "nme_unknown.png",
  "xp.png",
  "time.png",
  "tile.png",
  "boss.png",
  "heart.png",
  "road16.png",
  "town16.png",
  "scroll.png",
  "blocked.png",
];

pub const fn def(path: &'static str) -> Img {
  Img { path,
    rect: Rect{x: 0., y: 0., w: 128., h: 128. },
  }
}

pub const XP: Img = def("xp.png");
pub const TIME: Img = def("time.png");
pub const TILE: Img = def("tile.png");
pub const HERO: Img = def("hero.png");
pub const QUEST: Img = def("npc.png");
pub const UNKNOWN_ENEMY: Img = def("nme_unknown.png");
pub const SCROLL: Img = def("scroll.png");
pub const BLOCKED: Img = def("blocked.png");

pub const fn enemy_img(nme: EnemyType) -> Img {
  let path = match nme {
    EnemyType::Clyde  => "nme1.png",
    EnemyType::Blinky   => "nme2.png",
    EnemyType::Pinky => "nme3.png",
    EnemyType::GhostWitch  => "boss.png",
  };
  def(path)
}

pub const fn prize_img(prize: Prize) -> Img {
  let path = match prize {
    Prize::Heal => "heart.png",
  };
  def(path)
}


fn terrain_path(terrain: Terrain) -> &'static str {
  // TODO fill in filepaths for real terrain
  match terrain {
    Terrain::Road =>  "road16.png",
    Terrain::Grass => "terrain_grass.png",
    Terrain::Forest => "terrain_forest.png",
    Terrain::Town => "town16.png",
    _ =>  "terrain_placeholder.png",
  }
}

pub fn terrain16(terrain: Terrain, signature: [bool;4]) -> Img {
  let path = terrain_path(terrain);
  let mut x = 0.;
  let mut y = 0.;
  if signature[0] { x += 1.; }
  if signature[1] { x += 2.; }
  if signature[2] { y += 1.; }
  if signature[3] { y += 2.; }
  x *= 128.;
  y *= 128.;
  let rect = Rect { x, y, w: 128., h: 128. };
  Img { rect, path }
}

pub fn terrain_triangle(terrain: Terrain, d: Dir4) -> Img {
  let path = terrain_path(terrain);
  let i = d.index() as f32;
  let rect = Rect {
    x: i * 128.,
    y: 128.,
    w: 128.,
    h: 128.,
  };
  Img { rect, path, }
}

pub fn terrain_wedge(terrain: Terrain, d: Dir4) -> Img {
  let path = terrain_path(terrain);
  let i = d.index() as f32;
  let rect = Rect {
    x: i * 128.,
    y: 0.,
    w: 128.,
    h: 128.,
  };
  Img { rect, path }
}

pub fn terrain_bridge(terrain: Terrain, d: Dir4) -> Img {
  let path = terrain_path(terrain);
  let horizontal =  d == Dir4::Right ||
    d == Dir4::Left;
  let y = 128. * if horizontal {0.} else {1.};
  let rect = Rect {
    y,
    x: 128. * 4.,
    w: 128.,
    h: 128.,
  };
  Img { rect, path }
}

