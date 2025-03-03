
use std::cmp::Ordering;

use crate::*;

#[derive(Clone, Copy, Debug)]
struct ShadowInterval(IVec, IVec);
impl ShadowInterval {
  pub fn contains(&self, v: IVec) -> Contains {
    let o1 = orient2di(IVec::ZERO, v, self.0);
    let o2 = orient2di(IVec::ZERO, self.1, v);

    match (o1.cmp(&0), o2.cmp(&0)) {
        (Ordering::Equal, Ordering::Equal) => Contains::Boundary,
        (Ordering::Less, Ordering::Less) => Contains::Inside,
        _ => Contains::Outside,
    }
  }

  pub fn overlaps(&self, rhs: ShadowInterval) -> bool {
    [ self.contains(rhs.0),
      self.contains(rhs.1),
      rhs.contains(self.0),
      rhs.contains(self.1)
    ].into_iter().min().unwrap() <= Contains::Boundary
  }

  pub fn test_shadow(&self, rhs: ShadowInterval) -> Contains {
    let mut n = 0;
    if self.contains(rhs.0) <= Contains::Boundary { n += 1; }
    if self.contains(rhs.1) <= Contains::Boundary { n += 1; }
    match n {
      0 => Contains::Outside,
      1 => Contains::Boundary,
      _ => Contains::Inside,
    }
  }

  pub fn merge(&mut self, rhs: ShadowInterval) {
    if self.contains(rhs.0) == Contains::Outside { self.0 = rhs.0; }
    if self.contains(rhs.1) == Contains::Outside { self.1 = rhs.1; }
  }
}


#[repr(u8)]
#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
enum Contains {
  Inside,
  Boundary,
  Outside,
}

pub fn orient2di(p: IVec, q: IVec, r: IVec) -> i64 {
	let a = q - p;
	let b = r - p;
  (a.y as i64) * (b.x as i64) - (a.x as i64) * (b.y as i64)
}

#[derive(Debug)]
struct Shadows {
  intervals: Vec<ShadowInterval>,
}
impl Shadows {
  pub fn coverage(&self, s: ShadowInterval) -> usize {
    let mut ret = 0;
    for i in &self.intervals {
      match i.test_shadow(s) {
        Contains::Inside => return 3,
        Contains::Boundary => ret += 1,
        _ => {}
      }
    }
    ret
  }

  pub fn add_interval(&mut self, si: ShadowInterval) {
    let mut fused: Option<usize> = None;
    let mut i = 0;
    while i < self.intervals.len() {
      if si.overlaps(self.intervals[i]) {
        match fused {
          None => { // fuse si into the first overlap and mark it
            self.intervals[i].merge(si);
            fused = Some(i);
          }
          Some(j) => { // for 2+ overlaps, 
                       // take that interval out and fuse it into the first
            let ii = self.intervals.remove(i);
            self.intervals[j].merge(ii);
            // skip increment index since we removed a value at i
            continue;
          } 
        }
      }
      i += 1;
    }
    if fused.is_none() {
      self.intervals.push(si);
    }
  }
}

pub fn scan_quadrant<Pos>(
  from: Pos,
  rotation: i8,
  distance: u8,
  step: impl Fn(Pos, Dir4) -> Pos,
  blocked: impl Fn(Pos) -> bool,
  mut record: impl FnMut(Pos),
) 
  where
      Pos: Copy + Eq,
  {
    let right = Dir4::Right.rotate4(rotation);
    let up = Dir4::Up.rotate4(rotation);
    let origin = IVec::ZERO;
    let d2 = distance as i16 * 2;
    let mut shadows: Shadows = Shadows{ intervals: Vec::new() };
    let mut frontier = vec![(from, origin)];
    let mut next = vec![];
    while frontier.len() > 0 {
      for (pos,v) in frontier.drain(..) {
        // dont look at things that are too far back
        if v.distance_max(origin) > d2 {
          continue;
        }
        let cell_shadow = ShadowInterval(
            v + up.into() - right.into(),
            v + right.into() - up.into(),
        );

        // the two positions behind this in the scan
        let u = (step(pos,    up), v + 2*IVec::from(   up));
        let r = (step(pos, right), v + 2*IVec::from(right));


        let coverage = shadows.coverage(cell_shadow);
        if coverage >= 3 {
          continue;
        }
        
        record(pos);

        if blocked(pos)  && v != origin {
          shadows.add_interval(cell_shadow);
        }

        if coverage < 2 {
          if next.len() == 0 || *next.last().unwrap() != u { next.push(u); }
          next.push(r);
        }

      }

      std::mem::swap(&mut frontier, &mut next);
    }
  }


