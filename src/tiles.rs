
use crate::*;
use Terrain::*;

pub const TABLE: &[(usize, [Terrain;5])] = &[
  // basic terrain combos (forest, town, grass)
  // 4 of a kind
  (900, [Forest,Forest,Forest,Forest, None]),
  (900, [  Town,  Town,  Town,  Town, None]),
  (900, [ Grass, Grass, Grass, Grass, None]),
  // 3 of a kind
  (450, [Forest,Forest,Forest, Grass, None]),
  (450, [Forest,Forest,Forest,  Town, None]),
  (450, [Forest,  Town,  Town,  Town, None]),
  (450, [ Grass,  Town,  Town,  Town, None]),
  (900, [Forest, Grass, Grass, Grass, None]),
  (450, [ Grass, Grass, Grass,  Town, None]),
  // 2 of a kind (elbow, pair)
  (1000, [Forest,Forest, Grass, Grass, None]),
  (1000, [ Grass, Grass,  Town,  Town, None]),
  (800, [Forest,Forest,  Town,  Town, None]),
  // 2 of a kind (elbow, different)
  (100, [Forest,Forest, Grass,  Town, None]),
  (200, [Forest, Grass,  Town,  Town, None]),
  (100, [Forest, Grass, Grass,  Town, None]),
  // 2 of a kind (cross)
  ( 40, [Forest, Grass,Forest, Grass, Forest]),
  ( 60, [Forest, Grass,Forest, Grass, Grass]),
  ( 20, [Forest, Grass,Forest,  Town, Forest]),
  ( 30, [Forest, Grass,  Town, Grass, Grass]),
  ( 40, [Forest,  Town,Forest,  Town, Forest]),
  ( 40, [Forest,  Town,Forest,  Town, Town]),
  ( 40, [Forest,  Town, Grass,  Town, Town]),
  ( 60, [ Grass,  Town, Grass,  Town, Grass]),
  ( 40, [ Grass,  Town, Grass,  Town, Town]),

  // Rivers 4
  ( 10, [ River, River, River, River, None]),
  // Rivers 3
  ( 5, [ River, River, River,  Town, None]),
  ( 10, [ River, River, River, Grass, None]),
  ( 10, [ River, River, River,Forest, None]),
  ( 5, [ River, River, River,  Town, None]),
  // Rivers 1 -> _ 3
  ( 2, [ River,Forest,Forest,Forest, None]),
  ( 2, [ River, Grass, Grass, Grass, None]),
  ( 2, [ River,  Town,  Town,  Town, None]),
  // Rivers 1 -> _ 2 -> _ 1 (elbow)
  ( 6, [ River,Forest,Forest, Grass, None]),
  ( 2, [ River,Forest,Forest,  Town, None]),
  ( 8, [ River, Grass, Grass,Forest, None]),
  ( 4, [ River, Grass, Grass,  Town, None]),
  ( 4, [ River,  Town,  Town,Forest, None]),
  ( 6, [ River,  Town,  Town, Grass, None]),
  // Rivers 1 -> _ 2 -> _ 1 (bridge)
  ( 4, [ River, Grass,Forest, Grass, Grass]),
  ( 2, [ River, Grass,  Town, Grass, Grass]),
  ( 2, [ River,Forest, Grass,Forest, Forest]),
  ( 1, [ River,Forest,  Town,Forest, Forest]),
  ( 2, [ River,  Town,Forest,  Town, Town]),
  ( 2, [ River,  Town, Grass,  Town, Town]),
  // Rivers 2 -> _ 2 (elbow)
  (120, [ River, River,Forest,Forest, None]),
  (120, [ River, River, Grass, Grass, None]),
  (120, [ River, River,  Town,  Town, None]),
  // Rivers 2 -> _ 2 (bridge) (river cuts through)
  (120, [ River,Forest, River,Forest, River]),
  (120, [ River, Grass, River, Grass, River]),
  (120, [ River,  Town, River,  Town, River]),
  // Rivers 2 -> _ 1 -> _ 1 (elbow)
  (160, [ River, River,Forest,Grass, None]),
  ( 80, [ River, River,Forest, Town, None]),
  ( 80, [ River, River, Grass, Town, None]),
  // Rivers 2 -> _ 1 -> _ 1 (river cuts through)
  ( 80, [ River,Forest, River, Grass, River]),
  ( 40, [ River,Forest, River,  Town, River]),
  ( 40, [ River, Grass, River,  Town, River]),
  // Rivers 1 -> _ 1 ->  _ 1  -> _ 1
  (  3, [ River, Grass,Forest,   Town, None]),
  (  3, [ River, Grass,  Town, Forest, None]),
  (  3, [ River,Forest, Grass,   Town, None]),
  (  3, [ River,Forest,  Town,  Grass, None]),
  (  3, [ River,  Town,Forest,  Grass, None]),
  (  3, [ River,  Town, Grass, Forest, None]),


  // ROADS 4 
  ( 30, [Road,Road,Road,Road,None]),
  // ROADS 3 -> _1
  ( 60, [Road,Road,Road,Grass,None]),
  ( 15, [Road,Road,Road,Town,None]),
  ( 30, [Road,Road,Road,Forest,None]),
  ( 10, [Road,Road,Road,River,None]),

  // ROADS 2 -> _ 2 (elbow)
  (100, [Road,Road,Forest,Forest,None]),
  (200, [Road,Road,Grass,Grass,None]),
  (100, [Road,Road,Town,Town,None]),
  (100, [Road,Road,River,River,None]),
  // ROADS 2 -> _ 1 -> _1 (elbow)
  ( 80, [Road,Road,Forest,Grass,None]),
  ( 40, [Road,Road,Grass,Town,None]),
  ( 30, [Road,Road,Forest,Town,None]),
  ( 50, [Road,Road,Grass,River,None]),
  ( 30, [Road,Road,River,Town,None]),
  ( 10, [Road,Road,Forest,River,None]),

  // ROADS 2 -> _ 2 (bridge)
  ( 80, [Road,Forest,Road,Forest,Road]),
  (120, [Road,Grass,Road,Grass,Road]),
  ( 40, [Road,Town,Road,Town,Road]),
  ( 40, [Road,Town,Road,Town,Town]),
  ( 40, [Road,River,Road,River,Road]),
  // ROADS 2 -> _ 1 -> _1 (bridge)
  ( 50, [Road,Forest,Road,Grass,Road]),
  ( 10, [Road,Forest,Road,River,Road]),
  ( 20, [Road,Forest,Road, Town,Road]),
  ( 10, [Road, Grass,Road,River,Road]),
  ( 30, [Road, Grass,Road, Town,Road]),
  ( 10, [Road, River,Road, Town,Road]),


  // ROADS 1 -> _ 3
  ( 40, [Road,  Town,  Town,  Town,None]),
  ( 20, [Road, Grass, Grass, Grass,None]),
  ( 20, [Road,Forest,Forest,Forest,None]),
  ( 20, [Road, River, River, River,None]),
  // ROADS 1 -> _ 2 -> _ 1 (elbow)
  ( 30, [Road, Grass, Grass,Forest,None]),
  ( 20, [Road, Grass, Grass,  Town,None]),
  ( 20, [Road, Grass, Grass, River,None]),
  ( 20, [Road,Forest,Forest, Grass,None]),
  ( 10, [Road,Forest,Forest,  Town,None]),
  ( 10, [Road,Forest,Forest, River,None]),
  ( 10, [Road,  Town,  Town, Forest,None]),
  ( 20, [Road,  Town,  Town,  Grass,None]),
  ( 10, [Road,  Town,  Town,  River,None]),
  ( 10, [Road, River, River, Forest,None]),
  ( 20, [Road, River, River,  Grass,None]),
  (  5, [Road, River, River,   Town,None]),
  // ROADS 1 -> _ 2 -> _ 1 (bridge)
  ( 30, [Road, Grass,Forest, Grass, Grass]),
  ( 20, [Road, Grass,  Town, Grass, Grass]),
  ( 20, [Road, Grass, River, Grass, Grass]),
  ( 20, [Road,Forest, Grass,Forest,Forest]),
  (  5, [Road,Forest,  Town,Forest,Forest]),
  ( 10, [Road,Forest, River,Forest,Forest]),
  ( 10, [Road,  Town,Forest,  Town,  Town]),
  ( 20, [Road,  Town, Grass,  Town,  Town]),
  ( 10, [Road,  Town, River,  Town,  Town]),
  ( 10, [Road, River,Forest, River, River]),
  ( 20, [Road, River, Grass, River, River]),
  ( 10, [Road, River,  Town, River, River]),
  // ROADS 1 -> _ 1 -> _ 1 -> _ 1
  (  1, [  Road, Grass,  Town, River, None]),
  (  1, [  Road, Grass,  Town,Forest, None]),
  (  1, [  Road, Grass, River,  Town, None]),
  (  1, [  Road, Grass, River,Forest, None]),
  (  1, [  Road, Grass,Forest,  Town, None]),
  (  1, [  Road, Grass,Forest, River, None]),
  (  1, [  Road,  Town, Grass, River, None]),
  (  1, [  Road,  Town, Grass,Forest, None]),
  (  1, [  Road,  Town, River, Grass, None]),
  (  1, [  Road,  Town, River,Forest, None]),
  (  1, [  Road,  Town,Forest, Grass, None]),
  (  1, [  Road, River,  Town, Grass, None]),
  (  1, [  Road, River, Grass,  Town, None]),
  (  1, [  Road, River, Grass,Forest, None]),
  (  1, [  Road, River,Forest,  Town, None]),
  (  1, [  Road, River,Forest, Grass, None]),
  (  1, [  Road,Forest,  Town, Grass, None]),
  (  1, [  Road,Forest, Grass,  Town, None]),
  (  1, [  Road,Forest, Grass, River, None]),
  (  1, [  Road,Forest, River, Grass, None]),
  (  1, [  Road,  Town,Forest, River, None]),
  (  1, [  Road, River,  Town,Forest, None]),
  (  1, [  Road,Forest,  Town, River, None]),
  (  1, [  Road,Forest, River,  Town, None]),

];

// each tile is right up left down (matching dir4.index)
// [ 1 2 3 ]
// [ 4 5 6 ]
// [ 7 8 9 ]
pub fn boss_lair(rng: &mut Rng) -> [Tile;9] {
  let mut terrain_bag = [Road, River, Grass, Grass,
                         Grass, Grass, Grass, Grass,
                         Forest, Forest, Town, Town];
  shuffle(&mut terrain_bag, rng);
  [
  Tile { contents: [Road, terrain_bag[0], terrain_bag[1], Road, Road]},
  Tile { contents: [Road, terrain_bag[2], Road, Forest, Road]},
  Tile { contents: [terrain_bag[3], terrain_bag[4], Road, Road, Road]},
  Tile { contents: [Forest, Road, terrain_bag[5], Road,  Road]},
  Tile { contents: [Forest, Forest, Forest, Forest, None]},
  Tile { contents: [terrain_bag[6], Road, Forest, Road,  Road]},
  Tile { contents: [Road, Road, terrain_bag[7], terrain_bag[8], Road]},
  Tile { contents: [Road, Forest, Road, terrain_bag[9], Road]},
  Tile { contents: [terrain_bag[10], Road, Road, terrain_bag[11], Road]},
  ]
}

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
