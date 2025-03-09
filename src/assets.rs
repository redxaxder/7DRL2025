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
  "box.png",
  "boss.png",
  "heart.png",
  "road16.png",
  "town16.png",
  "scroll.png",
  "blocked.png",
  "river16.png",
  "arrows.png",
  "rothint.png",
  "path.png",
];


pub async fn load_sounds(res: &mut Resources) {
  pub const SOUNDS_TO_LOAD: &[&'static str] = &[
    "example.wav"
  ];
  for path in SOUNDS_TO_LOAD {
    res.load_sound(path).await;
  }
}



pub const fn def(path: &'static str) -> Img {
  Img { path,
    rect: Rect{x: 0., y: 0., w: 128., h: 128. },
  }
}

pub const XP: Img = def("xp.png");
pub const TIME: Img = def("time.png");
pub const TILE: Img = def("tile.png");
pub const BOX: Img = def("box.png");
pub const HERO: Img = def("hero.png");
pub const QUEST: Img = def("npc.png");
pub const UNKNOWN_ENEMY: Img = def("nme_unknown.png");
pub const SCROLL: Img = def("scroll.png");
pub const BLOCKED: Img = def("blocked.png");
pub const HEART: Img = def("heart.png");
pub const HINT: Img = def("rothint.png");


pub const fn path_img(dir: Dir4, arrow: bool) -> Img {
  let path = "path.png";
  let x = 128. * (dir.index() as f32);
  let y = if arrow {0.} else { 128.};
  let rect = Rect{x, y, w: 128., h: 128. };
  Img{ path, rect }
}


pub const fn arrow_img(d: Dir4) -> Img {
  let path = "arrows.png";
  let rect = Rect {
    x: (d.index() as u8 as f32) * 128.,
    y: 0.,
    w: 128.,
    h: 128.,
  };
  Img{path, rect}
}

pub const fn enemy_img(nme: EnemyType, alarmed: bool) -> Img {
  let path = match nme {
    EnemyType::Clyde  => "nme1.png",
    EnemyType::Blinky   => "nme2.png",
    EnemyType::Pinky => "nme3.png",
    EnemyType::GhostWitch  => "boss.png",
  };

  let rect = Rect {
    x: if alarmed {128.} else {0.},
    y: 0.,
    w: 128.,
    h: 128.,
  };
  Img{path, rect}
}

pub const fn prize_img(prize: Prize) -> Img {
  match prize {
    Prize::Heal => HEART,
  }
}


fn terrain_path(terrain: Terrain) -> &'static str {
  // TODO fill in filepaths for real terrain
  match terrain {
    Terrain::Road =>  "road16.png",
    Terrain::Grass => "terrain_grass.png",
    Terrain::Forest => "terrain_forest.png",
    Terrain::Town => "town16.png",
    Terrain::River => "river16.png",
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

