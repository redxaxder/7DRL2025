
use crate::Vec2;
use crate::Rng;

// IRect{{{
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub struct IRect {
  pub x: i16,
  pub y: i16,
  pub width: i16,
  pub height: i16,
}

fn wrap1(x: i16, min: i16, width: i16)  -> i16 {
  (x - min).rem_euclid(width) + min
}


impl IRect {
  pub fn origin(&self) -> IVec {
    IVec{x: self.x, y: self.y }
  }

  pub fn shift(self, v: IVec) -> IRect {
    IRect {
      x: self.x + v.x,
      y: self.y + v.y,
      width: self.width,
      height: self.height,
    }
  }
  pub fn intersection(self, rhs: IRect) -> IRect {
    let x = self.x.max(rhs.x);
    let y = self.y.max(rhs.y);
    let x1 = (self.x + self.width).min(rhs.x + rhs.width);
    let y1 = (self.y + self.height).min(rhs.y + rhs.height);
    IRect {
      x, y,
      width: 0.max(x1 - x),
      height: 0.max(y1 - y),
    }
  }

  pub fn contains(&self, position: Position) -> bool {
    position.x >= self.x
      && position.x < self.x + self.width
      && position.y >= self.y
      && position.y < self.y + self.height
  }

  pub fn clamp_pos(&self, position: Position) -> Position {
    IVec {
      x: position.x.clamp(self.x, self.x + self.width - 1),
      y: position.y.clamp(self.y, self.y + self.height - 1),
    }
  }

  pub fn wrap(&self, position: Position) -> Position {
    let x = wrap1(position.x, self.x, self.width);
    let y = wrap1(position.y, self.y, self.height);
    IVec {x,y}
  }

  pub fn size(&self) -> IVec {
    IVec{ x: self.width, y: self.height }
  }

  pub const fn linear_size(&self) -> usize {
    (self.width * self.height) as usize
  }

  pub fn to_linear_index(&self, pos: Position) -> usize {
    let px = pos.x - self.x;
    let py = pos.y - self.y;
    ((py * self.width) + px).try_into().unwrap_or(usize::MAX)
  }

  pub fn from_linear_index(&self, ix: usize) -> Position {
    let px = ix as i16 % self.width;
    let py = (ix as i16 - px) / self.width;
    let x = px + self.x;
    let y = py + self.y;
    IVec{x,y}
  }

  pub fn iter(&self) -> IRectIter {
    IRectIter {
      ix: 0,
      bound: self.linear_size(),
      rect: self,
    }
  }
}

pub struct IRectIter<'a> {
  ix: usize,
  bound: usize,
  rect: &'a IRect,
}

impl<'a> Iterator for IRectIter<'a> {
  type Item = IVec;
  fn next(&mut self) -> Option<IVec> {
    if self.ix >= self.bound { return None; }
    let v = self.rect.from_linear_index(self.ix);
    self.ix += 1;
    Some(v)
  }
}

impl<'a> DoubleEndedIterator for IRectIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
      if self.ix >= self.bound { return None; }
      let v = self.rect.from_linear_index(self.bound - self.ix - 1);
      self.ix += 1;
      Some(v)
    }
}



//}}}

// Dir8{{{
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum Dir8{
  Right = 0,
  UpRight = 1,
  Up = 2,
  UpLeft = 3,
  Left = 4,
  DownLeft = 5,
  Down = 6,
  DownRight = 7,
}

impl Dir8 {
  pub fn list() -> [Self;8] {
    unsafe {
      core::array::from_fn(|x| core::mem::transmute(x as u8))
    }
  }

  pub fn rotate8(&self, amount: i8) -> Self {
    let mut a = *self as i16;
    a += amount as i16;
    a &= 0b0111;
    unsafe{ std::mem::transmute(a as u8) }
  }

  pub fn is_primary(self) -> bool {
    self < self.opposite()
  }

  pub fn opposite(&self) -> Self {
    self.rotate8(4)
  }
}

impl From<u8> for Dir8 {
  fn from(x: u8) -> Self {
    unsafe{ std::mem::transmute(x%8) }
  }
}

impl From<Dir4> for Dir8 {
  fn from(x: Dir4) -> Self {
    unsafe{ std::mem::transmute(x) }
  }
}
//}}}

// Dir4{{{
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum Dir4{
  Right = 0,
  Up = 2,
  Left = 4,
  Down = 6,
}

impl Dir4 {
  pub const fn list() -> [Dir4; 4] {
    const LIST: [Dir4;4] = [Dir4::Right, Dir4::Up, Dir4::Left, Dir4::Down];
    LIST
  }

  pub fn is_primary(self) -> bool {
    self < self.opposite()
  }

  pub fn index(self) -> usize {
    (self as usize) >> 1
  }

  pub fn randlist(rng: &mut Rng) -> [Dir4; 4] {
    let mut u = rng.next_u64();
    let i = u & 0b11; //0123
    u = u >> 2;
    let k = (u & 1) + 2; //34
    u = u >> 1;
    let j = (u % 3) + 1; //123
    let mut result = Self::list();
    result.swap(0, i as usize);
    result.swap(1, j as usize);
    result.swap(3, k as usize);
    result
  }

  pub fn opposite(&self) -> Self {
    self.rotate4(2)
  }

  pub fn rotate4(&self, amount: i8) -> Self {
    let r = Dir8::from(*self).rotate8(amount * 2);
    r.try_into().unwrap()
  }
}

impl TryFrom<Dir8> for Dir4 {
  type Error = ();
  fn try_from(x: Dir8) -> Result<Self, ()> {
    if ((x as u8) & 1) == 1 {
      return Err(());
    }
    Ok(unsafe{ std::mem::transmute(x) })
  }
}
//}}}

// Position{{{
pub type Position = IVec;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct IVec {
  pub x: i16,
  pub y: i16
}
impl IVec {
  pub fn rounded(v: Vec2) -> Self {
    IVec {
      x: v.x.round_ties_even() as i16,
      y: v.y.round_ties_even() as i16,
    }
  }

  // x axis points right, y axis points up
  const DIRS: [IVec; 8] = [
    IVec{x:  1, y:  0},
    IVec{x:  1, y:  1},
    IVec{x:  0, y:  1},
    IVec{x: -1, y:  1},
    IVec{x: -1, y:  0},
    IVec{x: -1, y: -1},
    IVec{x:  0, y: -1},
    IVec{x:  1, y: -1},
  ];

  pub const ZERO: IVec = IVec{x: 0, y: 0};
  pub const ONE: IVec = IVec{x: 1, y: 1};


  // a random-ish number to use as a tiebreaker between posiitions
  // it makes the pattern:
  // 0 1 3 2 4 0 1 3 2 4 0 1 3 2 4
  // 2 4 0 1 3 2 4 0 1 3 2 4 0 1 3
  // 1 3 2 4 0 1 3 2 4 0 1 3 2 4 0
  // 4 0 1 3 2 4 0 1 3 2 4 0 1 3 2
  // 3 2 4 0 1 3 2 4 0 1 3 2 4 0 1
  // 0 1 3 2 4 0 1 3 2 4 0 1 3 2 4
  // 2 4 0 1 3 2 4 0 1 3 2 4 0 1 3
  // 1 3 2 4 0 1 3 2 4 0 1 3 2 4 0
  // 4 0 1 3 2 4 0 1 3 2 4 0 1 3 2
  // 3 2 4 0 1 3 2 4 0 1 3 2 4 0 1
  pub fn tiebreaker(&self) -> u8 {
    let a = (self.x + 2*(self.y)).rem_euclid(5) as u8;
    a ^ ((a & 2) >> 1)
  }

  pub fn distance1(self, rhs: IVec) -> i16 {
    let a = (self - rhs).abs();
    a.x + a.y
  }

  pub fn distance_max(self, rhs: IVec) -> i16 {
    let a = (self - rhs).abs();
    a.x.max(a.y)
  }

  pub fn abs(self) -> IVec {
    IVec {
      x: self.x.abs(),
      y: self.y.abs(),
    }
  }

}

impl From<IVec> for crate::Vec2 {
  fn from(v: IVec) -> Self {
    crate::Vec2{
      x: v.x.into(),
      y: v.y.into(),
    }
  }
}

impl From<Dir8> for IVec {
  fn from(x: Dir8) -> Self {
    IVec::DIRS[x as usize]
  }
}

impl From<Dir4> for IVec {
  fn from(x: Dir4) -> Self {
    IVec::DIRS[x as usize]
  }
}

impl From<Dir4> for Vec2 {
  fn from(x: Dir4) -> Self {
    IVec::DIRS[x as usize].into()
  }
}

impl From<i16> for IVec {
  fn from(value: i16) -> Self {
    IVec {
      x: value,
      y: value,
    }
  }
}


impl std::ops::Add<IVec> for IVec {
  type Output = Self;
  fn add(self, v: IVec) -> Self {
    IVec{
      x: self.x + v.x,
      y: self.y + v.y,
    }
  }
}

impl std::ops::AddAssign<IVec> for IVec {
  fn add_assign(&mut self, v: IVec) {
    *self = *self + v;
  }
}

impl std::ops::Sub<IVec> for IVec {
  type Output = Self;
  fn sub(self, v: IVec) -> Self {
    IVec{
      x: self.x - v.x,
      y: self.y - v.y,
    }
  }
}

impl std::ops::Mul<IVec> for IVec {
  type Output = Self;
  fn mul(self, v: IVec) -> Self {
    IVec{
      x: self.x * v.x,
      y: self.y * v.y,
    }
  }
}

impl std::ops::Mul<IVec> for i16 {
  type Output = IVec;
  fn mul(self, v: IVec) -> IVec {
    IVec{
      x: self * v.x,
      y: self * v.y,
    }
  }
}
impl From<(i16,i16)> for IVec {
  fn from(p: (i16,i16)) -> IVec {
    IVec{
      x: p.0,
      y: p.1,
    }
  }
}
//}}}

// Buffer2D{{{
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Buffer2D<T> {
  pub rect: IRect,
  pub contents: Vec<T>
}


impl<T> Buffer2D<T> {
  pub fn new(default: T, rect: IRect) -> Self  where
    T : Clone
  {
    let contents = vec![default; (rect.width * rect.height) as usize];
    Buffer2D { rect, contents }
  }

  pub fn fill(&mut self, item: T) where
    T: Clone {
      for c in self.contents.iter_mut() {
        *c = item.clone();
      }
  }

  pub fn get(&mut self, pos: Position) -> Option<&T> {
    if !self.rect.contains(pos) {
      return None;
    }
    self.contents.get(self.rect.to_linear_index(pos))
  }

  pub fn get_wrapped(&self, pos: Position) -> &T {
    self.contents.get(self.rect.to_linear_index(self.rect.wrap(pos))).unwrap()
  }

  pub fn get_mut(&mut self, pos: Position) -> Option<&mut T> {
    if !self.rect.contains(pos) {
      return None;
    }
    let ix = self.rect.to_linear_index(pos);
    self.contents.get_mut(ix)
  }

  pub fn get_wrapped_mut(&mut self, pos: Position) -> &mut T {
    self.contents.get_mut(self.rect.to_linear_index(self.rect.wrap(pos))).unwrap()
  }

  pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Buffer2D<U>
  {
    let rect = self.rect;
    let contents = self.contents.iter().map(f).collect();
    Buffer2D { rect, contents }
  }

  pub fn d8_action(&self, g: D8) -> Result<Self,()>
    where T: Clone,
  {
    if self.rect.width != self.rect.height {
      return Err(());
    }
    let mut next: Self = self.clone();
    let size = self.rect.width;
    let origin = self.rect.origin();

    for x in 0..self.rect.width {
      for y in 0..self.rect.height {
        let v = IVec{x,y};
        let src = origin + v;
        let tgt = origin + g.act_affine_square(v, size-1);
        next[tgt] = self[src].clone();

      }
    }
    Ok(next)

  }

}

impl<T: std::fmt::Display> std::fmt::Display for Buffer2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let mut y = None;
      for p in self.rect.iter() {
        if y != Some(p.y) {
          writeln!(f)?;
        }
        write!(f, "{}", self[p])?;
        y = Some(p.y);
      }
      Ok(())
    }
}

impl<T> std::ops::Index<IVec> for Buffer2D<T> {
  type Output = T;
  fn index(&self, pos: IVec) -> &T {
    self.get_wrapped(pos)
  }
}

impl<T> std::ops::IndexMut<IVec> for Buffer2D<T> {
  fn index_mut(&mut self, pos: IVec) -> &mut T {
    self.get_wrapped_mut(pos)
  }
}

//}}}

pub fn cardinal_alignment(from: Position, to: Position) -> Option<Dir4> {//{{{
  use core::cmp::Ordering;
  let delta = to - from;
  let dir = match (delta.x.cmp(&0), delta.y.cmp(&0)) {
    (Ordering::  Equal, Ordering::Greater) => Dir4::Up,
    (Ordering::  Equal, Ordering::   Less) => Dir4::Down,
    (Ordering::Greater, Ordering::  Equal) => Dir4::Right,
    (Ordering::   Less, Ordering::  Equal) => Dir4::Left,
    _ => return None,
  };
  Some(dir)
}//}}}

pub fn projectile_path(from: Position, dir: Dir4, blocked: &Buffer2D<bool>) -> Vec<Position> {//{{{
  let v = IVec::from(dir);
  let mut p = from + v;
  let mut result = vec![];
  while blocked.rect.contains(p) {
    result.push(p);
    if blocked[p] { break; }
    p += v;
  }
  result
}//}}}

// The Morton "Z-order" curve{{{
pub fn morton_curve(x: u32, y: u32) -> u64 {
  separate_bits(x) | (separate_bits(y) << 1)
}
fn separate_bits(x:u32) -> u64 {
  let mut r = x as u64;
  r = (r | r << 16) & 0x0000FFFF0000FFFF;
  r = (r | r << 8)  & 0x00FF00FF00FF00FF;
  r = (r | r << 4)  & 0x0F0F0F0F0F0F0F0F;
  r = (r | r << 2)  & 0x3333333333333333;
  r = (r | r << 1)  & 0x5555555555555555;
  r
}//}}}

#[repr(u8)]//{{{
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum D8 {
    E   = 0,
    R1  = 1,
    R2  = 2,
    R3  = 3,
    T   = 4,
    TR1 = 5,
    TR2 = 6,
    TR3 = 7,
}

impl D8 {
  pub fn list() -> [D8;8] {
    core::array::from_fn(|x| unsafe{ core::mem::transmute(x as u8) } )
  }

  #[inline]
  fn t(self) -> u8 {
    (self as u8) >> 2
  }
  #[inline]
  fn p(self) -> u8 {
    (self as u8) & 3
  }

  fn act_affine_square(self, v: IVec, size: i16) -> IVec {
    let mut result: IVec;
    let s = size;
    match self.p() {
      0 => result = v,
      1 => result = IVec{ x:     v.y, y: s - v.x },
      2 => result = IVec{ x: s - v.x, y: s - v.y },
      3 => result = IVec{ x: s - v.y, y:     v.x },
      _ => unreachable!(),
    }
    if self.t() > 0 {
      result.y = s - result.y;
    }
    result
  }


  fn act_dir8(self, d: Dir8) -> Dir8 {
    let mut r = d as u8;
    r += self.p() * 2;
    if self.t() > 0 {
      r = 16 - r;
    }
    r.into()
  }

  pub fn permute_array<T>(self, arr: &mut [T;4]) {
    arr.rotate_right(self.p() as usize);
    if self.t() > 0 {
      arr.swap(1,3);
    }
  }

  fn act_dir4(self, d: Dir4) -> Dir4 {
    self.act_dir8(d.into()).try_into().unwrap()
  }

  fn act_ivec(self, v: IVec) -> IVec {
    let IVec{mut x, mut y} = v;
    match self {
      D8::  E => {}
      D8:: R1 => { x = -v.y; y =  v.x; }
      D8:: R2 => { x = -v.x; y = -v.y; }
      D8:: R3 => { x =  v.y; y = -v.x; }
      D8::  T => {           y = -v.y; }
      D8::TR1 => { x = -v.y; y = -v.x; }
      D8::TR2 => { x = -v.x;           }
      D8::TR3 => { x =  v.y; y =  v.x; }
    }
    IVec{x,y}
  }

  const CAYLEY_TABLE: [[D8;8];8] = [
    /*               E       R1       R2       R3        T      TR1      TR2      TR3 */
    /*   E */ [D8::  E, D8:: R1, D8:: R2, D8:: R3, D8::  T, D8::TR1, D8::TR2, D8::TR3],
    /*  R1 */ [D8:: R1, D8:: R2, D8:: R3, D8::  E, D8::TR3, D8::  T, D8::TR1, D8::TR2],
    /*  R2 */ [D8:: R2, D8:: R3, D8::  E, D8:: R1, D8::TR2, D8::TR3, D8::  T, D8::TR1],
    /*  R3 */ [D8:: R3, D8::  E, D8:: R1, D8:: R2, D8::TR1, D8::TR2, D8::TR3, D8::  T],
    /*   T */ [D8::  T, D8::TR1, D8::TR2, D8::TR3, D8::  E, D8:: R1, D8:: R2, D8:: R3],
    /* TR1 */ [D8::TR1, D8::TR2, D8::TR3, D8::  T, D8:: R3, D8::  E, D8:: R1, D8:: R2],
    /* TR2 */ [D8::TR2, D8::TR3, D8::  T, D8::TR1, D8:: R2, D8:: R3, D8::  E, D8:: R1],
    /* TR3 */ [D8::TR3, D8::  T, D8::TR1, D8::TR2, D8:: R1, D8:: R2, D8:: R3, D8::  E],
  ];
}
//}}}

impl std::ops::Mul<Dir4> for D8 {//{{{
  type Output = Dir4;
  fn mul(self, rhs: Dir4) -> Dir4 {
    self.act_dir4(rhs)
  }
}//}}}

impl std::ops::Mul<Dir8> for D8 {//{{{
  type Output = Dir8;
  fn mul(self, rhs: Dir8) -> Dir8 {
    self.act_dir8(rhs)
  }
}//}}}

impl std::ops::Mul for D8 {//{{{
  type Output = Self;
  fn mul(self, rhs: D8) -> Self {
    D8::CAYLEY_TABLE[self as usize][rhs as usize]
  }
}//}}}

impl std::ops::Mul<IVec> for D8 {//{{{
  type Output = IVec;
  fn mul(self, rhs: IVec) -> IVec {
    self.act_ivec(rhs)
  }
}//}}}
