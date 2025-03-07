
use rl2025::*;


fn get_dist(terrain: Terrain) -> [usize;4] {
  let mut result = [0;4];
  for (weight, row) in tiles::TABLE {
    //result[(terrain, row)] += weight;

    let mut n = 0;

    let mut opposite = false;
    for i in 0..4 {
      if row[i] == terrain {
        n += 1; 
        opposite = opposite || row[(i+2) % 4] == terrain;
      }
    }
    if n == 0 { continue; }
    if n == 2 && opposite && row[4] != terrain && terrain != Terrain::River {
      result[0] += 2 * weight;
    } else {
      result[n-1] += weight;
    }

  }
  result
}


fn main() {
  for &t in Terrain::DRAW_ORDER {
    println!("================");
    println!("terrain {:?}", t);
    let dist = get_dist(t);
    let mut total = 0;
    let mut sides = 0;
    for i in 0..4 {
      total += dist[i];
      sides += (i+1) * dist[i];
    }
    println!("dist {:?}", dist);
    println!("avg {} / {} = {}", sides, total, sides as f64 / total as f64);
  }
 
}
