use crate::*;
use include_dir::{Dir, include_dir};

pub const ASSETS: Dir<'static> =  include_dir!("$CARGO_MANIFEST_DIR/assets");

pub const LOAD_ME: &[&'static str] = &[
  "hero.png",
  "terrain_placeholder.png",
  "npc.png",
  "nme1.png",
  "nme2.png",
  "nme3.png",
  "nme4.png",
  "nme_unknown.png",
  "xp.png",
];

pub const fn def(path: &'static str) -> Img {
  Img { path,
    rect: Rect{x: 0., y: 0., w: 128., h: 128. },
  }
}

pub const XP: Img = def("xp.png");
pub const HERO: Img = def("hero.png");
pub const QUEST: Img = def("npc.png");
pub const UNKNOWN_ENEMY: Img = def("nme_unknown.png");

pub const fn enemy(nme: EnemyType) -> Img {
  let path = match nme {
    EnemyType::Clyde  => "nme1.png",
    EnemyType::Blinky   => "nme2.png",
    EnemyType::Pinky => "nme3.png",
    EnemyType::GhostWitch  => "nme4.png",
    _ => "",
  };
  def(path)
}

fn terrain_path(terrain: Terrain) -> &'static str {
  // TODO fill in filepaths for real terrain
  match terrain {
    _ =>  "terrain_placeholder.png",
  }
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

