pub struct Multiset<T> {
  ts: Vec<T>,
  counts: Vec<u8>,
}

impl<T> Multiset<T> {
  pub fn new() -> Self {
    Multiset { ts: Vec::new(), counts: Vec::new() }
  }

  pub fn get(&self, t: &T) -> u8 where
    T: Eq + Ord
  {
    if let Ok(ix) = self.ts.binary_search(t) {
      self.counts[ix]
    }
    else { 0 }
  }

  pub fn insert(&mut self, t:T) where
    T: Eq + Ord
  {
    match self.ts.binary_search(&t) {
      Ok(ix) => self.counts[ix] += 1,
      Err(ix) => {
        self.ts.insert(ix, t);
        self.counts.insert(ix, 1);
      }
    }
  }
}


// impl<T> Ref<Vec<T>> {
//   pub fn push(&self, t: T) {
//     unsafe { self.get() }.push(t);
//   }
//   pub fn len(&self) -> usize {
//     self.peek().len()
//   }
// }






/*
// Set{{{
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Set<T>(pub Vec<T>);
impl<T> Set<T> {
  pub fn new() -> Self {
    Set(vec!())
  }
  pub fn from_vec(mut v: Vec<T>) -> Self where
    T: Ord + Eq
  {
    v.sort();
    v.dedup();
    Set(v)
  }

  pub fn insert(&mut self, t: T) where
    T: Ord + Eq
  {
    if let Err(ix) = self.0.binary_search(&t) {
      self.0.insert(ix, t);
    }
  }
  pub fn remove(&mut self, t: &T) where
    T: Ord + Eq
  {
    if let Ok(ix) = self.0.binary_search(t) {
      self.0.remove(ix);
    }
  }
  pub fn contains(&self, t: &T) -> bool where
    T: Ord + Eq
  {
    self.0.binary_search(t).is_ok()
  }
  pub fn iter(&self) -> std::slice::Iter<'_, T> {
    self.0.iter()
  }
  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn union(mut self, mut rhs: Set<T>) -> Set<T>  where
    T: Ord + Eq
  {
    self.0.append(&mut rhs.0);
    self.0.sort();
    self.0.dedup();
    self
  }

  pub fn minus(mut self, rhs: Set<T>) -> Set<T> where
    T: Ord + Eq
  {
    let mut i = 0;
    let xs = self.0.drain(..).filter(|x| {
       while i < rhs.len() && *x > rhs[i] {
         i += 1;
       }
       if i >= rhs.len() {
         true
       } else {
         *x != rhs[i]
       }
    }).collect();
    Set(xs)
  }

}


impl<T> std::ops::Index<usize> for Set<T> {
  type Output = T;
  fn index(&self, i: usize) -> &T {
    &self.0[i]
  }
}//}}}
*/

pub type Set<T> = linear_map::set::LinearSet<T>;
pub type Map<K,V> = linear_map::LinearMap<K,V>;
