
#[derive(Debug)]
struct Gadget {
  a: u32,
  b: u32,
}
impl Gadget {
  fn getb(&mut self) -> DropBear<u32> {
    let a: &mut u32 = &mut self.a;
    let ptr = &mut self.b;
    let cleanup = Box::new( |&mut final_b| {
      *a = final_b
    });
    DropBear { ptr, cleanup }
  }
}

struct DropBear<'a, T> {
  ptr: &'a mut T,
  cleanup: Box<dyn FnOnce(&'a mut T) + 'a>,
}

impl<'a, T> Drop for DropBear<'a,T> {
  fn drop(&mut self) {
    let mut f:Box <dyn FnOnce(&'a mut T) + 'a> = Box::new(|_|{});
    std::mem::swap(&mut self.cleanup, &mut f);
    let p = self.ptr as *mut T;
    f(unsafe { &mut *p })
  }
}

use std::ops::Deref;
impl<'a,T> Deref for DropBear<'a,T> {
  type Target = T;
  fn deref(&self) -> &T {
    &self.ptr
  }
}

use std::ops::DerefMut;
impl<'a,T> DerefMut for DropBear<'a,T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.ptr
  }
}


fn main() {
  let mut gadget = Gadget{a: 1, b:2};
  println!("{:?}", &gadget);

  {
    let mut bb = gadget.getb();
    *bb = 3;
  }
  println!("{:?}", &gadget);

}
