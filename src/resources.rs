
use std::collections::HashMap;
use crate::*;
use include_dir::Dir;

type Path = &'static str;


pub struct Resources {
  pub assets: Dir<'static>,
  pub textures: HashMap<Path, Texture2D>,
}

impl Resources {
  pub fn new(assets: Dir<'static>) -> Self {
    Resources {
      assets,
      textures: HashMap::new(),
    }
  }


  pub fn load_texture(
    &mut self,
    path: Path,
    filter: FilterMode
    ) -> &Texture2D {
    if !self.textures.contains_key(path) {
      let file = self.assets.get_file(path).expect(&format!("missing {}", path));
      let t: Texture2D = Texture2D::from_file_with_format(
        file.contents(), None
      );
      t.set_filter(filter);
      self.textures.insert(path, t);
    }
    self.textures.get(path).unwrap()
  }

  pub fn draw_image(&self, x: f32, y: f32, w: f32, h: f32, rotation: f32, color: Color, image: &Img) {
    if let Some(atlas) = self.textures.get(image.path) {
      draw_texture_ex(
        atlas,
        x,
        y,
        color,
        DrawTextureParams {
          dest_size: Some(Vec2{x:w,y:h}),
          source: Some(image.rect),
          rotation: -rotation,
          ..Default::default()
        }
      );
    } else {
      warn!("missing source image");
    }
  }
}

#[derive(Debug, Clone)]
pub struct Img {
  pub path: Path,
  pub rect: Rect,
}

pub const fn index_rect(tile: u8, row_len: u8, dim: IVec) -> Rect {
  let i = tile % row_len;
  let j = tile / row_len;
  let w = dim.x;
  let h = dim.y;
  Rect{
    x: (i * w as u8) as f32,
    y: (j * h as u8) as f32,
    w: w as f32,
    h: h as f32,
  }
}
