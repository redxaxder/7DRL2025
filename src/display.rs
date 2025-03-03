
use crate::*;


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
      (self.bounds.height as f32 - u.y) - (self.bounds.y as f32),
    ).into();
    v * full_tile
  }

}


pub struct Display {
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
    Display {
      resources,
      render_to,
      texture,
      dim,
    }
  }

  pub fn dim(&self) -> Vec2 {
    self.dim
  }

  pub fn draw_tile(&self,
    position: ScreenCoords,
    grid: &Grid,
    color: Color,
    image: &Img,
    ) {
    self.resources.draw_image(
      position.x + grid.tile_margin.x,
      position.y + grid.tile_margin.y,
      grid.tile_size.x,
      grid.tile_size.y,
      0.,
      color,
      image,
    );
  }

  pub fn draw_tile_rotated(&self,
    position: ScreenCoords,
    grid: &Grid,
    rotation: f32,
    color: Color,
    image: &Img,
    ) {
    self.resources.draw_image(
      position.x + grid.tile_margin.x,
      position.y + grid.tile_margin.y,
      grid.tile_size.x,
      grid.tile_size.y,
      rotation,
      color,
      image,
    );
  }

}


pub type ScreenCoords = Vec2;



