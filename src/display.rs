
use crate::*;


pub const CAMERA_TETHER: IRect = IRect {x: -1, y: -1, width: 4, height: 3 };


pub const DISPLAY_GRID: Grid = Grid {
  bounds: IRect{ x: -6, y: -5, width: 12, height: 10},
  tile_size: Vec2{ x: 128., y: 128. },
  tile_margin: Vec2::ZERO,
};

pub struct Grid {
  pub bounds: IRect,
  pub tile_size: Vec2,
  pub tile_margin: Vec2,
}
impl Grid {
  pub fn dim(&self) -> Vec2 {
    Vec2::from(self.bounds.size()) * self.full_tile_size()
  }

  pub fn full_tile_size(&self) -> Vec2 {
    self.tile_size + 2.*self.tile_margin
  }

  pub fn to_screen(&self, u: Vec2) -> ScreenCoords {
    let full_tile = self.tile_size +(2. * self.tile_margin);
    let v: Vec2 = (
      u.x - (self.bounds.x as f32),
      (self.bounds.height as f32 - u.y) + (self.bounds.y as f32),
    ).into();
    v * full_tile
  }


  pub fn rect(&self, u: impl Into<Vec2>) -> Rect {
    let p = DISPLAY_GRID.to_screen(u.into());
    Rect {
      x: p.x,
      y: p.y,
      w: self.tile_size.x,
      h: self.tile_size.y,
    }
  }



}


pub struct Display {
  pub camera_focus: IVec,
  pub resources: Resources,
  pub render_to: Camera2D,
  pub texture: Texture2D,
  pub dim: Vec2,
}


impl Display {
  pub fn new(resources: Resources, dim: Vec2) -> Self {
    let buffer = render_target(dim.x as u32, dim.y as u32);
    buffer.texture.set_filter(FilterMode::Nearest);
    let render_to = {
      let mut x = Camera2D::from_display_rect( Rect::new(
          0., 0., dim.x as f32, dim.y as f32
      ));
      x.render_target = Some(buffer.clone());
      x
    };
    let texture = buffer.texture.clone();
    let camera_focus = IVec::ZERO;

    Self{ camera_focus, resources, render_to, texture, dim, }
  }

  pub fn camera_wrap_bounds(&self) -> Rect {
    let x: f32 = (self.camera_focus.x-BOARD_RECT.width/2) as f32 * DISPLAY_GRID.full_tile_size().x;
    let y: f32 = (self.camera_focus.y-BOARD_RECT.height/2) as f32 * DISPLAY_GRID.full_tile_size().y;
    let w = BOARD_RECT.width as f32 * DISPLAY_GRID.full_tile_size().x;
    let h = BOARD_RECT.height as f32 * DISPLAY_GRID.full_tile_size().y;
    Rect { x, y, w, h }
  }

  pub fn draw_img(&self,
    rect: Rect,
    color: Color,
    image: &Img,
  ) {
    self.resources.draw_image(
      rect.x,
      rect.y,
      rect.w,
      rect.h,
      0.,
      color,
      image,
    );
  }

  pub fn draw_grid(&self,
    position: Vec2,
    color: Color,
    image: &Img,
  ) {
    let rect = DISPLAY_GRID.rect(position - Vec2::from(self.camera_focus));
    self.draw_img(rect, color, image);
  }

  pub fn draw_tile(&self, rect: Rect, tile: Tile) {
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
          self.draw_img(rect, terrain.color(), &img);
        }
      } else if opposite && tile.contents[4] == terrain {
        // no adjacency + opposite + center implies bridge
        for d in Dir4::list() {
          if tile.contents[d.index()] != terrain { continue; }
          let img = terrain_bridge(terrain, d);
          self.draw_img(rect, terrain.color(), &img);
          break; // a single bridge image covers both directions
        }
      } else {
        // fallthrough is wedge
        for d in Dir4::list() {
          if tile.contents[d.index()] != terrain { continue; }
          let img = terrain_wedge(terrain, d);
          self.draw_img(rect, terrain.color(), &img);
        }

      }
      // TODO:
      // draw special center item if present
      // eg quest
    }
  }
}

pub type ScreenCoords = Vec2;



