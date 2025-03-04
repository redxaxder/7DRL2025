use std::collections::hash_map::DefaultHasher;
use core::hash::{Hash, Hasher};
use std::convert::TryInto;


pub fn from_global_rng() -> Rng {
  Rng::new(
    crate::rand::rand() as u64 | ((crate::rand::rand() as u64) << 32),
    crate::rand::rand() as u64 | ((crate::rand::rand() as u64) << 32),
  )
}

pub fn from_current_time() -> Rng {
  let now:f64 = macroquad::miniquad::date::now();
  let state: u64 = unsafe { std::mem::transmute(now) };
  Rng::new(state, 0)
}

// Pcg32 copied from rand_pcg
// but without the extra deps i don't need and don't want to wrangle build for
#[derive(Clone)]
pub struct Rng {
    state: u64,
    increment: u64,
}
impl Rng {
  pub fn new(state: u64, increment: u64) -> Self {
    Rng { state, increment }
  }

  #[inline]
  pub fn next_u32(&mut self) -> u32 {
    let state = self.state;
    self.step();

    // Output function XSH RR: xorshift high (bits), followed by a random rotate
    // Constants are for 64-bit state, 32-bit output
    const ROTATE: u32 = 59; // 64 - 5
    const XSHIFT: u32 = 18; // (5 + 32) / 2
    const SPARE: u32 = 27; // 64 - 32 - 5

    let rot = (state >> ROTATE) as u32;
    let xsh = (((state >> XSHIFT) ^ state) >> SPARE) as u32;
    xsh.rotate_right(rot)
  }

  #[inline]
  pub fn next_u64(&mut self) -> u64 {
    ((self.next_u32() as u64) << 32) | self.next_u32() as u64
  }

  #[inline]
  fn step(&mut self) {
    const MULTIPLIER: u64 = 6364136223846793005;
    // prepare the LCG for the next round
    self.state = self
      .state
      .wrapping_mul(MULTIPLIER)
      .wrapping_add(self.increment);
  }

}

pub fn scale_u32_pow(x: u32, mut exp: usize) -> u32 {
  let mut base = x as u64;
  let mut result = 1 << 32;
  while exp > 0 {
    if exp & 1 > 0 {
      result *= base;
      result = result >> 32
    }
    base = (base * base) >> 32;
    exp = exp >> 1;
  }
  result as u32
}

// calculates a power of x through repeated squaring
// as if x was a fixed point value in [0,1]
pub fn scale_u64_pow(x: u64, mut exp: usize) -> u64 {
  let mut base = x as u128;
  let mut result = u64::MAX as u128;
  while exp > 0 {
    if exp & 1 > 0 {
      result *= base;
      result = result >> 64
    }
    base = (base * base) >> 64;
    exp = exp >> 1;
  }
  result as u64
}

pub trait HashRandom {
    fn rng_stream(&self, s: u64) -> Rng;
    fn rng(&self) -> Rng {
        self.rng_stream(0)
    }
}

impl<T: Hash> HashRandom for T {
    fn rng_stream(&self, s: u64) -> Rng {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let res = hasher.finish();
        Rng::new(res, s)
    }
}

pub fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
  let n = v.len();
  for i in 0..n {
    let j = i + ((rng.next_u32() as usize) % (n-i));
    v.swap(i,j);
  }
}

// shuffles an array of indices based on weights
// if i has weight A and j has weight B, i appears before j with odds A:B
pub fn shuffle_ixs_weighted(weights: &[usize], rng: &mut Rng) -> Vec<usize> {
  let n = weights.len();
  let mut ixs: Vec<usize> = Vec::with_capacity(n);
  let mut rolls: Vec<u64> = Vec::with_capacity(n);
  for i in 0..n {
    rolls.push(
      scale_u64_pow(rng.next_u64(), weights[i])
    );
    ixs.push(i);
  }
  ixs.sort_by_key(|i| rolls[*i]);
  ixs
}

// shuffles an array of values with weights
// if i has weight A and j has weight b, i appears before j with odds A:B
pub fn weighted_shuffle<T>(v: &mut [T], weights: &[usize], rng: &mut Rng) {
  assert!(v.len() == weights.len());
  let mut ixs = shuffle_ixs_weighted(weights, rng);
  apply_permutation(v, &mut ixs);
}

// rearranges the contents of an array to match a permutation
// destroys the permutation in the process
// p[i] = j ----> xs[i] ends up in position j
pub fn apply_permutation<T>(xs: &mut [T], permutation: &mut [usize]) {
  assert!(xs.len() == permutation.len());
  let n = xs.len();
  for i in 0..n {
    while i != permutation[i] {
      let j = permutation[i];
      xs.swap(i,j);
      permutation.swap(i,j);
    }
  }
}

// pick N items from a set of size m
pub fn subseq_indices<const N: usize>(m: usize, rng: &mut Rng) -> [usize;N] {
  assert!(N <= m);
  let mut picked: Vec<usize> = Vec::with_capacity(N);
  let mut i = 0;
  while i < N {
    let mut p: usize = (rng.next_u32() as usize ) % (m-i);
    for x in picked.iter() {
      if *x <= p { p += 1 }
    }
    picked.push(p);
    picked.sort();
    i+=1;
  }
  picked.try_into().unwrap()
}

pub fn subseq_copy<T, const N: usize>(values: &[T], rng: &mut Rng) -> [T;N]
  where T: Copy
{
  let mut result = [values[0];N];
  let indices = subseq_indices::<N>(values.len(), rng);
  for i in 0..N {
    result[i] = values[indices[i]];
  }
  result
}




#[derive(Debug)]
pub struct AliasTable {
  total_weight: usize,
  sample_threshold: Vec<usize>,
  overflow: Vec<usize>,
}
impl AliasTable {
  pub fn new(weights: &[usize]) -> Self {
    let len = weights.len();
    let total_weight: usize = weights.iter().sum();
    let mut sample_threshold = vec![0; len];
    let mut overflow: Vec<usize> = (0..len).collect();
    let mut small = Vec::new();
    let mut large = Vec::new();
    let mut scaled_weights: Vec<usize> = weights.iter().map(|&w| w * len).collect();
    for (i, &weight) in scaled_weights.iter().enumerate() {
      if weight <= total_weight {
        small.push(i);
      } else {
        large.push(i);
      }
    }
    while let Some(small_idx) = small.pop() {
      sample_threshold[small_idx] = scaled_weights[small_idx];
      if scaled_weights[small_idx] == total_weight { continue; }
      let large_idx = large.pop().unwrap();
      overflow[small_idx] = large_idx;

      scaled_weights[large_idx] -= total_weight - scaled_weights[small_idx];
      if scaled_weights[large_idx] <= total_weight {
        small.push(large_idx);
      } else {
        large.push(large_idx);
      }
    }
    for idx in large.into_iter().chain(small) {
      sample_threshold[idx] = total_weight;
    }
    AliasTable { total_weight, sample_threshold, overflow }
  }

  pub fn len(&self) -> usize {
    self.sample_threshold.len()
  }

  pub fn sample(&self, rng: &mut Rng) -> usize {
    let slot = rng.next_u32() as usize % self.len();
    let roll = rng.next_u32() as usize % self.total_weight;
    if roll < self.sample_threshold[slot] {
      slot
    } else {
      self.overflow[slot]
    }
  }

}


