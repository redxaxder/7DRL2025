
use crate::*;
use Terrain::*;

const TABLE: &[(usize, [Terrain;5])] = &[
  (5, [Grass,Grass,Grass,Grass,None]),
  (5, [Town,Town,Town,Town,None]),
  (5, [Forest,Forest,Forest,Forest,None]),
  (5, [River,River,Town,Town,None]),
  (5, [Road,Forest,Road,Grass,Road]),
  (5, [Forest,Grass,Grass,Town,None]),
];

const fn weight() -> usize {
  let mut r = 0;
  let n = TABLE.len();
  let mut i = 0;
  while i < n {
    let (w,_) = TABLE[i];
    r += w;
    i += 1;
  }
  r
}

const TABLE_WEIGHT: usize = weight();

pub fn generate(rng: &mut Rng) -> Tile {
  let mut w = rng.next_u64() as usize % TABLE_WEIGHT;
  let mut i = 0;
  while w > TABLE[i].0 {
    w -= TABLE[i].0;
    i += 1;
  }
  let g: D8 = D8::list()[rng.next_u64() as usize % 8];
  g * Tile { contents : TABLE[i].1 }

}


