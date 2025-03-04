
use crate::*;
use Terrain::*;

const TABLE: &[(usize, [Terrain;5])] = &[
  // basic terrain combos (forest, town, grass)
  // 4 of a kind
  ( 50, [Forest,Forest,Forest,Forest, None]),
  ( 50, [  Town,  Town,  Town,  Town, None]),
  (100, [ Grass, Grass, Grass, Grass, None]),
  // 3 of a kind
  (150, [Forest,Forest,Forest, Grass, None]),
  (150, [Forest,Forest,Forest,  Town, None]),
  (150, [Forest,  Town,  Town,  Town, None]),
  (150, [ Grass,  Town,  Town,  Town, None]),
  (300, [Forest, Grass, Grass, Grass, None]),
  (300, [ Grass, Grass, Grass,  Town, None]),
  // 2 of a kind (elbow)
  (500, [Forest,Forest, Grass, Grass, None]),
  (500, [ Grass, Grass,  Town,  Town, None]),
  (400, [Forest,Forest,  Town,  Town, None]),
  (300, [Forest,Forest, Grass,  Town, None]),
  (300, [Forest, Grass,  Town,  Town, None]),
  (300, [Forest, Grass, Grass,  Town, None]),
  // 2 of a kind (cross)
  ( 40, [Forest, Grass,Forest, Grass, Forest]),
  ( 60, [Forest, Grass,Forest, Grass, Grass]),
  ( 40, [Forest, Grass,Forest,  Town, Forest]),
  ( 60, [Forest, Grass,  Town, Grass, Grass]),
  ( 40, [Forest,  Town,Forest,  Town, Forest]),
  ( 40, [Forest,  Town,Forest,  Town, Town]),
  ( 40, [Forest,  Town, Grass,  Town, Town]),
  ( 60, [ Grass,  Town, Grass,  Town, Grass]),
  ( 40, [ Grass,  Town, Grass,  Town, Town]),

  // Rivers 4
  ( 10, [ River, River, River, River, None]),
  // Rivers 3
  ( 10, [ River, River, River,  Town, None]),
  ( 10, [ River, River, River, Grass, None]),
  ( 10, [ River, River, River,Forest, None]),
  ( 10, [ River, River, River,  Town, None]),
  // Rivers 1 -> _ 3
  ( 10, [ River,Forest,Forest,Forest, None]),
  ( 10, [ River, Grass, Grass, Grass, None]),
  ( 10, [ River,  Town,  Town,  Town, None]),
  // Rivers 1 -> _ 2 -> _ 1 (elbow)
  ( 20, [ River,Forest,Forest, Grass, None]),
  ( 20, [ River,Forest,Forest,  Town, None]),
  ( 20, [ River, Grass, Grass,Forest, None]),
  ( 20, [ River, Grass, Grass,  Town, None]),
  ( 20, [ River,  Town,  Town,Forest, None]),
  ( 20, [ River,  Town,  Town, Grass, None]),
  // Rivers 1 -> _ 2 -> _ 1 (bridge)
  ( 10, [ River,Forest, Grass,Forest, Forest]),
  ( 10, [ River,Forest,  Town,Forest, Forest]),
  ( 20, [ River, Grass,Forest, Grass, Grass]),
  ( 20, [ River, Grass,  Town, Grass, Grass]),
  ( 10, [ River,  Town,Forest,  Town, Town]),
  ( 10, [ River,  Town, Grass,  Town, Town]),
  // Rivers 2 -> _ 2 (elbow)
  ( 30, [ River, River,Forest,Forest, None]),
  ( 30, [ River, River, Grass, Grass, None]),
  ( 30, [ River, River,  Town,  Town, None]),
  // Rivers 2 -> _ 2 (bridge) (river cuts through)
  ( 20, [ River,Forest, River,Forest, River]),
  ( 20, [ River, Grass, River, Grass, River]),
  ( 20, [ River,  Town, River,  Town, River]),
  // Rivers 2 -> _ 1 -> _ 1 (elbow)
  ( 40, [ River, River,Forest,Grass, None]),
  ( 40, [ River, River,Forest, Town, None]),
  ( 40, [ River, River, Grass, Town, None]),
  // Rivers 2 -> _ 1 -> _ 1 (river cuts through)
  ( 20, [ River,Forest, River, Grass, River]),
  ( 20, [ River,Forest, River,  Town, River]),
  ( 20, [ River, Grass, River,  Town, River]),
  // Rivers 1 -> _ 1 ->  _ 1  -> _ 1
  (  5, [ River, Grass,Forest,   Town, None]),
  (  5, [ River, Grass,  Town, Forest, None]),
  (  5, [ River,Forest, Grass,   Town, None]),
  (  5, [ River,Forest,  Town,  Grass, None]),
  (  5, [ River,  Town,Forest,  Grass, None]),
  (  5, [ River,  Town, Grass, Forest, None]),


  // ROADS 4 
  ( 30, [Road,Road,Road,Road,None]),
  // ROADS 3 -> _1
  ( 60, [Road,Road,Road,Grass,None]),
  ( 30, [Road,Road,Road,Town,None]),
  ( 30, [Road,Road,Road,Forest,None]),
  ( 10, [Road,Road,Road,River,None]),

  // ROADS 2 -> _ 2 (elbow)
  ( 50, [Road,Road,Forest,Forest,None]),
  (100, [Road,Road,Grass,Grass,None]),
  ( 50, [Road,Road,Town,Town,None]),
  ( 50, [Road,Road,River,River,None]),
  // ROADS 2 -> _ 1 -> _1 (elbow)
  ( 80, [Road,Road,Forest,Grass,None]),
  ( 80, [Road,Road,Grass,Town,None]),
  ( 50, [Road,Road,Forest,Town,None]),
  ( 50, [Road,Road,Grass,River,None]),
  ( 50, [Road,Road,River,Town,None]),
  ( 10, [Road,Road,Forest,River,None]),

  // ROADS 2 -> _ 2 (bridge)
  ( 40, [Road,Forest,Road,Forest,Road]),
  ( 60, [Road,Grass,Road,Grass,Road]),
  ( 20, [Road,Town,Road,Town,Road]),
  ( 20, [Road,Town,Road,Town,Town]),
  ( 40, [Road,River,Road,River,Road]),
  // ROADS 2 -> _ 1 -> _1 (bridge)
  ( 50, [Road,Forest,Road,Grass,Road]),
  ( 10, [Road,Forest,Road,River,Road]),
  ( 30, [Road,Forest,Road, Town,Road]),
  ( 10, [Road, Grass,Road,River,Road]),
  ( 50, [Road, Grass,Road, Town,Road]),
  ( 10, [Road, River,Road, Town,Road]),


  // ROADS 1 -> _ 3
  ( 40, [Road,  Town,  Town,  Town,None]),
  ( 20, [Road, Grass, Grass, Grass,None]),
  ( 20, [Road,Forest,Forest,Forest,None]),
  ( 20, [Road, River, River, River,None]),
  // ROADS 1 -> _ 2 -> _ 1 (elbow)
  ( 30, [Road, Grass, Grass,Forest,None]),
  ( 30, [Road, Grass, Grass,  Town,None]),
  ( 20, [Road, Grass, Grass, River,None]),
  ( 20, [Road,Forest,Forest, Grass,None]),
  ( 10, [Road,Forest,Forest,  Town,None]),
  ( 10, [Road,Forest,Forest, River,None]),
  ( 10, [Road,  Town,  Town, Forest,None]),
  ( 20, [Road,  Town,  Town,  Grass,None]),
  ( 10, [Road,  Town,  Town,  River,None]),
  ( 10, [Road, River, River, Forest,None]),
  ( 20, [Road, River, River,  Grass,None]),
  ( 10, [Road, River, River,   Town,None]),
  // ROADS 1 -> _ 2 -> _ 1 (bridge)
  ( 30, [Road, Grass,Forest, Grass, Grass]),
  ( 30, [Road, Grass,  Town, Grass, Grass]),
  ( 20, [Road, Grass, River, Grass, Grass]),
  ( 20, [Road,Forest, Grass,Forest,Forest]),
  ( 10, [Road,Forest,  Town,Forest,Forest]),
  ( 10, [Road,Forest, River,Forest,Forest]),
  ( 10, [Road,  Town,Forest,  Town,  Town]),
  ( 20, [Road,  Town, Grass,  Town,  Town]),
  ( 10, [Road,  Town, River,  Town,  Town]),
  ( 10, [Road, River,Forest, River, River]),
  ( 20, [Road, River, Grass, River, River]),
  ( 10, [Road, River,  Town, River, River]),
  // ROADS 1 -> _ 1 -> _ 1 -> _ 1
  (  3, [  Road, Grass,  Town, River, None]),
  (  3, [  Road, Grass,  Town,Forest, None]),
  (  3, [  Road, Grass, River,  Town, None]),
  (  3, [  Road, Grass, River,Forest, None]),
  (  3, [  Road, Grass,Forest,  Town, None]),
  (  3, [  Road, Grass,Forest, River, None]),
  (  3, [  Road,  Town, Grass, River, None]),
  (  3, [  Road,  Town, Grass,Forest, None]),
  (  3, [  Road,  Town, River, Grass, None]),
  (  3, [  Road,  Town, River,Forest, None]),
  (  3, [  Road,  Town,Forest, Grass, None]),
  (  3, [  Road, River,  Town, Grass, None]),
  (  3, [  Road, River, Grass,  Town, None]),
  (  3, [  Road, River, Grass,Forest, None]),
  (  3, [  Road, River,Forest,  Town, None]),
  (  3, [  Road, River,Forest, Grass, None]),
  (  3, [  Road,Forest,  Town, Grass, None]),
  (  3, [  Road,Forest, Grass,  Town, None]),
  (  3, [  Road,Forest, Grass, River, None]),
  (  3, [  Road,Forest, River, Grass, None]),
  (  2, [  Road,  Town,Forest, River, None]),
  (  2, [  Road, River,  Town,Forest, None]),
  (  2, [  Road,Forest,  Town, River, None]),
  (  2, [  Road,Forest, River,  Town, None]),

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
