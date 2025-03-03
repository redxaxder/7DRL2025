
impl IntoIterator for DLX {
    type Item = Vec<usize>;

    type IntoIter = DLXIterator;

    fn into_iter(self) -> Self::IntoIter {
        DLXIterator::init(self)
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DLX {
  pub header: usize,
  pub rows:   usize,
  pub left:   Vec<usize>,
  pub right:  Vec<usize>,
  pub up:     Vec<usize>,
  pub down:   Vec<usize>,
  pub size:   Vec<usize>,
  pub column: Vec<usize>,
  pub row:    Vec<usize>,
}

impl DLX {
  pub fn init(cols: usize) -> Self {
    Self::init_aux(cols, 0)
  }

  pub fn init_aux(required_cols: usize, auxiliary_cols: usize) -> Self {
    let total_cols = required_cols + auxiliary_cols;
    let header = total_cols;
    let rows = 0;

    let mut left = vec![header; total_cols+1]; // item to the left of me
    let mut right = vec![header; total_cols+1]; // item to the right of me
    let mut up = vec![header; total_cols+1];    // item above
    let mut down = vec![header; total_cols+1];  // item below
    let size = vec![0; total_cols+1];       // how many 1s are in this column
    let mut column = vec![header; total_cols+1]; // what is the id of the column this is in
    let row = vec![header; total_cols+1]; // what row is this in

    // init header
    left[header] = required_cols-1;
    right[header] = 0;

    // init columns
    for c in 0..total_cols {
      column[c] = c;
      down[c] = c;
      up[c] = c;
      left[c] = if c == 0 {
        header
      } else if c < required_cols {
        c - 1
      } else { 
        c
      };
      right[c] = if c == (required_cols-1) {
        header
      } else if c < required_cols {
        c+1
      } else {
        c
      };
    }

    let dlx = DLX {
      header, rows,
      column: column.into(),
      row:    row.into(),
      size:   size.into(),
      left:   left.into(),
      right:  right.into(),
      up:     up.into(),
      down:   down.into(),
    };

    dlx
  }

  pub fn append_dense_row(&mut self, row: &[u8]) -> bool {
    if row.len() != self.header {
      println!("jank! {:?}", row);
      return false;
    }

    let mut scratch = vec![];
    for i in 0..self.header {
      if row[i] > 0 {
        scratch.push(i);
      }
    }
    self.append_row(&scratch)
  }

  pub fn export(&self) -> Vec<Vec<usize>> {
    let mut rows = vec!();
    rows.push(vec![]);
    for item in 0..self.column.len() {
      if item <= self.header { continue; }
      let r = self.row[item];
      if r >= rows.len() {
        rows.push(vec!());
      }
      rows[r].push(self.column[item]);
    }
    rows
  }

  pub fn append_row(&mut self, row: &[usize]) -> bool {
    let row_id = self.rows;
    let mut row_start = None;
    for &col in row {
      if col >= self.header {
        println!("jank! {:?} ({} >= {})", row, col, self.header);
        return false; 
      }
    }

    for &col in row {
      self.size[col] += 1;
      let item_id = self.column.len();
      self.column.push(col);
      self.row.push(row_id);
      self.up.push(self.up[col]);
      self.down.push(col);
      self.up[col] = item_id;
      self.down[self.up[item_id]] = item_id;

      if let Some(rs) = row_start {
        self.left.push(self.left[rs]);
        self.right.push(rs);
        self.left[rs] = item_id;
        self.right[self.left[item_id]] = item_id;
      }
      else {
        row_start = Some(item_id);
        self.left.push(item_id);
        self.right.push(item_id);
      }
    }

    self.rows += 1;
    true
  }

  pub fn display(&self) {
    let mut col = self.header;
    while let Some(next_col) = step_cursor(col, &self.right, self.header) {
      col = next_col;
      print!("col {}: [{}] ", col, self.size[col]);
      let mut item = col;
      while let Some(next_item) = step_cursor(item, &self.down, col) {
        item = next_item;
        print!("{} ", item);
      }
      println!();
    }

  }

  fn cover_column(&mut self, item: usize) {
    let column = self.column[item];
    // println!("cover {}", column);
    // self.display();
    self.left[self.right[column]] = self.left[column];
    self.right[self.left[column]] = self.right[column];
    let mut i = column;
    while let Some(next_row) = step_cursor(i, &self.down, column) {
      i = next_row;
      self.hide_row(i);
    }
    // println!(">>");
    // self.display();
    // println!("===================");
  }
  fn uncover_column(&mut self, item: usize) {
    let column = self.column[item];

    // println!("uncover {}", column);
    // self.display();

    let mut i = column;
    while let Some(next_row) = step_cursor(i, &self.up, column) {
      i = next_row;
      self.unhide_row(i);
    }
    self.left[self.right[column]] = column;
    self.right[self.left[column]] = column;

    // println!(">>");
    // self.display();
    // println!("===================");

  }

  fn hide_row(&mut self, item: usize) {
    // println!("hide row [{}] {} ", self.row[item], item);
    let mut j = item;
    while let Some(next_item) = step_cursor(j, &self.right, item) {
      j = next_item;
      self.up[self.down[j]] = self.up[j];
      self.down[self.up[j]] = self.down[j];
      self.size[self.column[j]] -= 1;
    }
  }
  fn unhide_row(&mut self, item: usize) {
    // println!("unhide row [{}] {} ", self.row[item], item);
    let mut j = item;
    while let Some(next_item) = step_cursor(j, &self.left, item) {
      j = next_item;
      self.size[self.column[j]] += 1;
      self.up[self.down[j]] = j;
      self.down[self.up[j]] = j;
    }
  }

  pub fn into_iter(self) -> DLXIterator {
    DLXIterator::init(self)
  }

  pub fn search(self) -> Option<Vec<usize>> {
    self.into_iter().next()
  }

}


pub struct DLXIterator {
  pub core: DLX,
  pub selected_rows: Vec<usize>,
  pub stack: Vec<StackFrame>
}

#[derive(Debug)]
struct StackFrame {
  min_col_size: usize,
  current_col: usize,
  current_row: usize,
}

impl Iterator for DLXIterator {
  type Item = Vec<usize>;
  fn next(&mut self) -> Option<Self::Item> {
    self.get_next().map(|x| x.into())
  }
}

impl DLXIterator {
  fn get_min_col_size(&self) -> Option<usize> {
    let mut c = self.core.header;
    let mut it = usize::MAX;
    loop {
      c = self.core.right[c];
      if c == self.core.header { break; }
      it = it.min(self.core.size[c]);
    }
    if it <= usize::MAX {
      Some(it)
    } else {
      None
    }
  }

  fn down(&self, row: usize) -> Option<usize> {
    let n = self.core.down[row];
    if self.core.column[n] == n {
      None
    } else {
      Some(n)
    }
  }

  fn next_column(&self, column: usize, column_size: usize) -> Option<usize> {
    let mut c = column;
    loop {
      c = self.core.right[c];
      if c == self.core.header {
        return None;
      }
      if self.core.size[c] == column_size {
        return Some(c);
      }
    }
  }

  pub fn check(&self) -> bool {
    // println!("[CHECK]");
    let h = self.core.header;
    let mut i = h;
    let mut open = vec![];
    loop {
      i = self.core.right[i];
      if i == h { break; }
      open.push(i);
    }
    // println!("OPEN {:?}", open);
    // self.core.display();
    for i in open {
      if !self.cuci(i) {
        // println!();
        return false;
      }
    }
    // println!("[CHECK DONE]");
    true
  }

  pub fn cuci(&self, column: usize) -> bool {
    let mut cc = self.core.clone();
    assert!(cc == self.core);
    cc.cover_column(column);
    cc.uncover_column(column);
    cc == self.core
  }

  fn cover_column(&mut self, column: usize) {
    self.core.cover_column(column);
  }

  fn uncover_column(&mut self, column: usize) {
    self.core.uncover_column(column);
  }

  fn select_row(&mut self, row: usize) {
    // println!("select row {}", self.core.row[row]);
    self.selected_rows.push(self.core.row[row]);
    let mut r = row;
    loop {
      r = self.core.right[r];
      if r == row { break; }
      self.cover_column(r);
    }
  }

  fn deselect_row(&mut self, row: usize) {
    // println!("deselect row {}", self.core.row[row]);
    let mut r = row;
    loop {
      r = self.core.left[r];
      if r == row { break; }
      self.uncover_column(r);
    }
    self.selected_rows.pop();
  }


  fn get_next<'a>(&'a mut self) -> Option<&'a [usize]> {
    while self.stack.len() > 0 {
      self.step(false);
      if self.core.right[self.core.header] == self.core.header {
        return self.current()
      }
    }
    None
  }

  pub fn current(&self) -> Option<&[usize]> {
    if self.stack.len() > 0 {
      Some(&self.selected_rows)
    } else {
      None
    }
  }


  pub fn next_forward<'a>(&'a mut self) -> Option<&'a [usize]> {
    self.step(false);
    self.current()
  }

  pub fn next_backtrack<'a>(&'a mut self) -> Option<&'a [usize]> {
    self.step(true);
    self.current()
  }

  pub fn init(core: DLX) -> Self {
    // println!("init dlxiter");
    // println!("{:?}", core);
    let mut x = Self {
      core, stack: vec![],
      selected_rows: vec![],
    };
    if let Some(min_col_size) = x.get_min_col_size() {
      if let Some(current_col) = x.next_column(
        x.core.header,
        min_col_size,
      ) {
        if let Some(current_row) = x.down(current_col) {
          x.cover_column(current_col);
          x.select_row(current_row);
          x.stack.push(StackFrame {
            min_col_size,
            current_col,
            current_row,
          });
        }
      }

    }
    x
  }

  fn step(&mut self, mut backtracked: bool) {
    while let Some(mut frame) = self.stack.pop() {
      if !backtracked {
        // first try to recurse deeper
        if let Some(min_col_size) = self.get_min_col_size() {
          if let Some(current_col) = self.next_column(self.core.header, min_col_size) {
            if let Some(current_row) = self.down(current_col) {
              self.cover_column(current_col);
              self.select_row(current_row);

              let f = StackFrame {
                min_col_size,
                current_col,
                current_row
              };
              self.stack.push(frame);
              self.stack.push(f);
              return;
            }
          }
        }
      }
      // We can't go deeper; try to advance to the next row
      self.deselect_row(frame.current_row);
      if let Some(next_row) = self.down(frame.current_row) {
        self.select_row(next_row);
        frame.current_row = next_row;
        self.stack.push(frame);
        return;
      }
      // We can't to advance the row; try to advance to the next column
      self.uncover_column(frame.current_col);
      if let Some(next_column) = self.next_column(
        frame.current_col,
        frame.min_col_size,
      ) {
        if let Some(next_row) = self.down(next_column) {
          self.cover_column(next_column);
          self.select_row(next_row);
          frame.current_row = next_row;
          frame.current_col = next_column;
          self.stack.push(frame);
          return;
        }
      }
      // We couldn't advance the column either. We drop the frame on the floor
      // and pick back the previous one at the start of the loop.
      // The backtracked flag prevents us from immediately reentering this one.
      backtracked = true;
    }
  }
}

fn step_cursor(item: usize, next: &[usize], stop:usize) -> Option<usize> {
  let n = next[item];
  if n == stop {
    None
  } else {
    Some(n)
  }
}
