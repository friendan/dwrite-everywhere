#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub struct UnsafeSendSync<T>(T);

impl<T> UnsafeSendSync<T> {
  pub unsafe fn new(value: T) -> Self {
    UnsafeSendSync(value)
  }

  pub fn into_inner(self) -> T {
    self.0
  }

  pub fn as_ref(&self) -> &T {
    &self.0
  }

  pub fn as_ref_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

unsafe impl<T> Send for UnsafeSendSync<T> {}
unsafe impl<T> Sync for UnsafeSendSync<T> {}
