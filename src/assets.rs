use crate::*;
use include_dir::{Dir, include_dir};

pub const ASSETS: Dir<'static> =  include_dir!("$CARGO_MANIFEST_DIR/assets");


pub const LOAD_ME: &[&'static str] = &[
  "hero.png",
  "terrain_placeholder.png",
];


pub const HERO: &Img = &Img {
  path: "hero.png",
  rect: Rect{x: 0., y: 0., w: 128., h: 128. },
};

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

