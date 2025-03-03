use std::collections::VecDeque;
use crate::*;

#[derive(Clone, Copy)]
pub struct AnimLock{
  require: u128,
  reserve: u128,
}

pub trait Lock {
  fn to_lock(self) -> u128;
}

impl AnimLock {
  pub fn empty() -> Self {
    AnimLock {
      require: 0,
      reserve: 0,
    }
  }


  pub fn requires(locs: impl Lock) -> Self {
    Self::empty().require(locs)
  }

  pub fn reserves(locs: impl Lock) -> Self {
    Self::empty().reserve(locs)
  }

  // is a pair of locks claiming the same resource?
  pub fn overlaps(&self, other: Self) -> bool {
    (self.require & other.reserve > 0)
      || (self.reserve & other.require > 0)
  }

  pub fn merge(&self, other: Self) -> Self {
    AnimLock {
      require: self.require | other.require,
      reserve: self.reserve | other.reserve,
    }
  }

  pub fn full() -> Self {
    AnimLock {
      require: u128::MAX,
      reserve: u128::MAX,
    }
  }


  pub fn require(mut self, req: impl Lock) -> Self {
    self.require |= req.to_lock(); //Self::pack_locations(BOUNDS, locs);
    self
  }

  // pub fn require_id(mut self, id: u64) -> Self {
  //   self.require |= Self::pack_id(id);
  //   self
  // }

  pub fn reserve(mut self, reserve: impl Lock) -> Self {
    let l = reserve.to_lock();
    self.require |= l;
    self.reserve |= l;
    self
  }

  // pub fn reserve_id(mut self, id: u64) -> Self {
  //   self.require |= Self::pack_id(id);
  //   self.reserve |= Self::pack_id(id);
  //   self
  // }


  // fn from_locations(rect: IRect, locs: &[IVec]) -> Self {
  //   let x = Self::pack_locations(rect, locs);
  //   AnimLock{
  //     require: x,
  //     reserve: x,
  //   }
  // }
}

pub fn pack_location(loc: IVec) -> u128 {
  let mut x = 0;
  let px = (loc.x) as u32;
  let py = (loc.y) as u32;
  let bit = morton_curve(px,py) % 64;
  x = x | (1 << bit);
  x
}

pub fn pack_id(id: impl Into<u64>) -> u128 {
  1 << ((id.into() % 64) + 64)
}


impl Lock for u64 {
  fn to_lock(self) -> u128 {
    pack_id(self)
  }
}

impl Lock for &u64 {
  fn to_lock(self) -> u128 {
    pack_id(*self)
  }
}

impl Lock for IVec {
  fn to_lock(self) -> u128 {
    pack_location(self)
  }
}
impl Lock for &IVec {
  fn to_lock(self) -> u128 {
    pack_location(*self)
  }
}

impl<T:Lock + Copy> Lock for &Vec<T> {
  fn to_lock(self) -> u128 {
    let mut it = 0;
    for x in self.iter() {
      it |= x.to_lock()
    }
    it
  }
}
impl<T:Lock + Copy> Lock for &[T] {
  fn to_lock(self) -> u128 {
    let mut it = 0;
    for x in self {
      it |= x.to_lock()
    }
    it
  }
}
impl<T:Lock + Copy, const N: usize> Lock for &[T;N] {
  fn to_lock(self) -> u128 {
    let mut it = 0;
    for x in self {
      it |= x.to_lock()
    }
    it
  }
}

impl std::ops::BitOrAssign<AnimLock> for AnimLock {
  fn bitor_assign(&mut self, other: AnimLock) {
    *self = self.merge(other)
  }
}

//impl From<Event> for AnimLock {
//  fn from(it: Event) -> Self {
//    match it {
//      Event::UnitMoved{from, to, ..} => {
//        Self::from_locations(BOUNDS, &[from, to])
//      }
//      Event::Intent{from, ..} => {
//        Self::from_locations(BOUNDS, &[from])
//      }
//      Event::SyncPoint => {
//        Self::full()
//      }
//    }
//  }
//}
//
//
// impl<const N: usize> From<[Position;N]> for AnimLock {
//   fn from(ps: [Position;N]) -> Self {
//     Self::from_locations(BOUNDS, &ps)
//   }
// }
// 
// impl From<&[Position]> for AnimLock {
//   fn from(ps: &[Position]) -> Self {
//     Self::from_locations(BOUNDS, ps)
//   }
// }
// 
// impl From<Unit> for AnimLock {
//   fn from(it: Unit) -> Self {
//     Self::from_locations(BOUNDS, &[it.pos])
//   }
// }

impl <T1,T2> From<(T1,T2)> for AnimLock where
  T1: Into<AnimLock>,
  T2: Into<AnimLock>,
{
  fn from(xs: (T1,T2)) -> Self {
    let mut x = AnimLock::empty();
    x = x.merge(xs.0.into());
    x = x.merge(xs.1.into());
    x
  }
}


pub struct AnimationQueue {
  timestamp: Seconds,
  animations: VecDeque<Animation>,
}

pub fn empty_animation(_t: Time) -> bool {
  false
}

impl AnimationQueue {
  pub fn new() -> Self {
    AnimationQueue {
      timestamp: get_time(),
      animations: VecDeque::new(),
    }
  }

  pub fn clear(&mut self) {
    self.animations.clear();
  }

  pub fn sync(&mut self) {
    let a = self.append(empty_animation);
    a.lock = AnimLock::full();
  }

  pub fn append(
    &mut self,
    func: impl FnMut(Time) -> bool  + 'static
  ) -> &mut Animation {
    self.animations.push_back( Animation {
      lock: AnimLock::empty(),
      elapsed: 0.,
      f: Box::new(func),
      state: AnimationState::Waiting,
      speed: 1.
    });
    self.animations.back_mut().unwrap()
  }

  pub fn len(&self) -> usize {
    self.animations.len()
  }

  pub fn hurry(&mut self, c: f64) {
    for a in self.animations.iter_mut() {
      a.speed *= c;
    }
  }

  pub fn tick(&mut self) {
    let now = get_time();
    let delta = now - self.timestamp;
    self.timestamp = now;

    let mut lock: AnimLock = AnimLock::empty();
    let mut can_chain = true;

    let mut i = 0;
    while i < self.animations.len() {
      let a = &mut self.animations[i];
      a.try_wake(lock, can_chain);
      a.tick(delta);
      let is_finished = a.state == AnimationState::Finished;
      can_chain = is_finished;
      if !is_finished { lock |= a.lock; }
      if is_finished && i == 0 {
        self.animations.pop_front();
      } else {
        i += 1;
      }
    }
  }
}

pub struct Time {
  pub delta: Seconds,
  pub elapsed: Seconds,
}
impl Time {
  pub fn progress(&self, duration: Seconds) -> f32 {
    f64::clamp(self.elapsed / duration, 0., 1.) as f32
  }
}


pub type Func = Box<dyn FnMut(Time) -> bool>;

pub struct Animation {
  pub lock: AnimLock,
  f: Func,
  elapsed: Seconds,
  state: AnimationState,
  speed: f64,
}

impl Animation {
  pub fn tick(&mut self, delta: Seconds) {
    if self.state == AnimationState::Active {
      self.elapsed += delta * self.speed;
      let elapsed = self.elapsed;
      let alive = (self.f)(Time{ delta, elapsed });
      if !alive { self.state = AnimationState::Finished; }
    }
  }

  pub fn require(&mut self, locs: impl Lock) -> &mut Self {
    self.lock = self.lock.require(locs); self
  }


  pub fn reserve(&mut self, locs: impl Lock) -> &mut Self  {
    self.lock = self.lock.reserve(locs); self
  }

  //pub fn require_id(&mut self, id: u64) -> &mut Self  {
  //  self.lock = self.lock.require_id(id); self
  //}
  //pub fn reserve_id(&mut self, id: u64) -> &mut Self  {
  //  self.lock = self.lock.reserve_id(id); self
  //}


  pub fn chain(&mut self) -> &mut Self {
    self.state = AnimationState::Chain;
    self
  }

  pub fn try_wake(&mut self, conflicts: AnimLock, has_chain: bool) {
    match self.state {
      AnimationState::Waiting => {
        if !self.lock.overlaps(conflicts) {
          self.state = AnimationState::Active
        }
      }
      AnimationState::Chain => {
        if !self.lock.overlaps(conflicts) && has_chain {
          self.state = AnimationState::Active
        }
      }
      _ => {}
    }
  }

}

#[derive(Clone,Copy,Eq,PartialEq)]
enum AnimationState {
  Waiting,
  Chain,
  Active,
  Finished,
}

