

use std::rc::Rc;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Clone)]
pub struct Ref<T>{
  rc: Rc<UnsafeCell<T>>,
  // so that this is not inferred as Sync or Send
  _marker: PhantomData<*const ()>,
}
impl<T> Ref<T> {
  pub unsafe fn get(&self) -> &mut T {
    let cell: &UnsafeCell<T> = &*self.rc;
    let p: *mut T = cell.get();
    &mut *p
  }
  pub fn peek(&self) -> &T {
    unsafe {
      let cell: &UnsafeCell<T> = &*self.rc;
      let p: *mut T = cell.get();
      & *p
    }
  }
  pub fn new(t: T) -> Self {
    Ref{
      rc: Rc::new(UnsafeCell::new(t)),
      _marker: PhantomData,
    }
  }

}

impl<T> std::ops::Deref for Ref<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.get() }
  }
}

impl<T> std::ops::DerefMut for Ref<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.get() }
  }
}

impl<T> From<T> for Ref<T> {
  fn from(value: T) -> Self {
    Ref::new(value)
  }
}


impl<T: Index<Ix>, Ix> Index<Ix> for Ref<T> 
{
  type Output = <T as Index<Ix>>::Output;

  fn index(&self, index: Ix) -> &Self::Output {
    &self.peek()[index]
  }
}
impl<T: IndexMut<Ix>, Ix> IndexMut<Ix> for Ref<T> 
{
  fn index_mut(&mut self, index: Ix) -> &mut Self::Output {
    unsafe {
      let m = self.get();
      &mut m[index]
    }
  }
}

