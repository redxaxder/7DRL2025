
use crate::*;


// why is this asymmetrical over x/y?
// i don't know. i just shifted it instead of investigating.
// i guess somehow related to y flip stuff
pub const CAMERA_TETHER: IRect = IRect {x: -1, y: -2, width: 4, height: 4 };


pub const DISPLAY_GRID: Grid = Grid {
  bounds: IRect{ x: -5, y: -5, width: 10, height: 10},
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
    Vec2::from(self.bounds.size()) * (self.tile_size + 2.*self.tile_margin)
  }

  pub fn to_screen(&self, u: Vec2) -> ScreenCoords {
    let full_tile = self.tile_size +(2. * self.tile_margin);
    let v: Vec2 = (
      u.x - (self.bounds.x as f32),
      (self.bounds.height as f32 - u.y) + (self.bounds.y as f32),
    ).into();
    v * full_tile
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
  pub fn draw_grid(&self,
    position: Vec2,
    color: Color,
    image: &Img,
  ) {
    let p = DISPLAY_GRID.to_screen(position - Vec2::from(self.camera_focus));
    self.resources.draw_image(
      p.x,
      p.y,
      DISPLAY_GRID.tile_size.x,
      DISPLAY_GRID.tile_size.y,
      0.,
      color,
      image,
    );
  }
}


pub type ScreenCoords = Vec2;



