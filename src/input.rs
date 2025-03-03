
use crate::*;
use Dir4::*;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum Input {
  Pass,
  InputDir(Dir4),
  InputAction(u8),
  Confirm,
  Reset,
  Cancel,
}

pub fn get_input() -> Option<Input> {
  let map = input_map();
  for &key in get_keys_pressed().iter() {
    if let Ok(found) = map.binary_search_by_key(&(key as u16), |x| x.0 as u16) {
      return Some(map[found].1);
    }
  }
  None
}

pub fn input_map() -> &'static [(KeyCode, Input)] {
  unsafe{
    if !INPUT_MAP_SORTED {
      INPUT_MAP.sort_by_key(|x| { x.0 as u16 });
      INPUT_MAP_SORTED = true;
    }
    core::ptr::addr_of!(INPUT_MAP)
      .as_ref().unwrap()
  }
}

static mut INPUT_MAP_SORTED:bool = false;
static mut INPUT_MAP: [(KeyCode, Input); 12] = [
  (KeyCode::Key1, Input::InputAction(0)),
  (KeyCode::Key2, Input::InputAction(1)),
  (KeyCode::Key3, Input::InputAction(2)),
  (KeyCode::Key3, Input::InputAction(3)),
  (KeyCode::W, Input::InputDir(Up)),
  (KeyCode::A, Input::InputDir(Left)),
  (KeyCode::S, Input::InputDir(Down)),
  (KeyCode::D, Input::InputDir(Right)),
  (KeyCode::X, Input::Pass),
  (KeyCode::E, Input::Confirm),
  (KeyCode::Q, Input::Reset),
  (KeyCode::Z, Input::Cancel),
];



