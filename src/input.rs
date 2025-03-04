
use crate::*;
use Dir4::*;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum Input {
  Dir(Dir4),
  Rotate1,
  Rotate2,
  Discard,
  LevelUp,
}

static mut INPUT_MAP: &mut [(KeyCode, Input)] = &mut [
  (KeyCode::W, Input::Dir(Up)),
  (KeyCode::A, Input::Dir(Left)),
  (KeyCode::S, Input::Dir(Down)),
  (KeyCode::D, Input::Dir(Right)),
  (KeyCode::Q, Input::Rotate1),
  (KeyCode::E, Input::Rotate2),
  (KeyCode::Z, Input::LevelUp),
  (KeyCode::X, Input::Discard),

  (KeyCode::K, Input::Dir(Up)),
  (KeyCode::H, Input::Dir(Left)),
  (KeyCode::J, Input::Dir(Down)),
  (KeyCode::L, Input::Dir(Right)),
  (KeyCode::Y, Input::Rotate1),
  (KeyCode::U, Input::Rotate2),
  (KeyCode::B, Input::LevelUp),
  (KeyCode::N, Input::Discard),

  (KeyCode::Up, Input::Dir(Up)),
  (KeyCode::Left, Input::Dir(Left)),
  (KeyCode::Down, Input::Dir(Down)),
  (KeyCode::Right, Input::Dir(Right)),
  (KeyCode::Comma, Input::Rotate1),
  (KeyCode::Period, Input::Rotate2),
  (KeyCode::Enter, Input::LevelUp),
  (KeyCode::Backspace, Input::Discard),
];

pub fn get_input() -> Option<Input> {
  let map = input_map();
  for &key in get_keys_pressed().iter() {
    if let Ok(found) = map.binary_search_by_key(&(key as u16), |x| x.0 as u16) {
      return Some(map[found].1);
    }
  }
  None
}

static mut INPUT_MAP_SORTED:bool = false;
pub fn input_map() -> &'static [(KeyCode, Input)] {
  unsafe{
    if !INPUT_MAP_SORTED {
    #[allow(static_mut_refs)]
      INPUT_MAP.sort_by_key(|x| { x.0 as u16 });
      INPUT_MAP_SORTED = true;
    }
    core::ptr::addr_of!(INPUT_MAP)
      .as_ref().unwrap()
  }
}
