use winapi::um::winnt::HRESULT;

#[derive(Debug, Fail)]
#[fail(display = "{}:{}:{} {}", file, line, col, error)]
pub struct Annotated<E> {
  pub error: E,
  pub file: &'static str,
  pub line: u32,
  pub col: u32,
}

pub type HResult<T> = ::std::result::Result<T, Annotated<HRESULT>>;

#[macro_export]
macro_rules! annotate_error {
  ($e:expr) => {
    $crate::errors::Annotated {
      error: $e,
      file: file!(),
      line: line!(),
      col: column!(),
    }
  };
}

#[macro_export]
macro_rules! com_invoke {
  ($obj:ident.$fun:ident, $($arg:tt),*) => { com_invoke!(@imp [$obj.$fun] $($arg),*) };
  (($($fun:tt)*), $($arg:tt),*) => { com_invoke!(@imp [$($fun)*] $($arg),*) };

  (@imp [$($fun:tt)*] $($arg:tt),*) => {
    {
      $(com_invoke!(@init $arg);)*;

      #[allow(unused_parens, unused_unsafe)]
      let hr = unsafe { $($fun)*($(com_invoke!(@arg $arg),)*) };
      if ::winapi::shared::winerror::SUCCEEDED(hr) {
        #[allow(unused_unsafe)]
        Ok(com_invoke!(@ret $($arg,)*))
      } else {
        Err($crate::errors::Annotated {
          error: hr,
          file: file!(),
          line: line!(),
          col: column!(),
        })
      }
    }
  };

  (@init (-> $a:ident)) => { #[allow(unused_unsafe)] let mut $a = unsafe { ::std::mem::uninitialized() }; };
  (@init (->> $a:ident)) => { #[allow(unused_unsafe)] let mut $a = unsafe { ::std::mem::uninitialized() }; };
  (@init $a:expr) => { () };

  (@arg (-> $a:ident)) => { &mut $a };
  (@arg (->> $a:ident)) => { &mut $a };
  (@arg $a:expr) => { $a };

  (@ret) => { () };
  (@ret (-> $a:ident), $($arg:tt,)*) => { ($a, com_invoke!(@ret $($arg,)*)) };
  (@ret (->> $a:ident), $($arg:tt,)*) => { (unsafe { ::wio::com::ComPtr::from_raw($a) }, com_invoke!(@ret $($arg,)*)) };
  (@ret $a:expr, $($arg:tt,)*) => { com_invoke!(@ret $($arg,)*) };
}
