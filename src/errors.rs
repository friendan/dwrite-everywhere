use winapi::um::winnt::HRESULT;

#[derive(Debug, Fail)]
#[fail(display = "{}:{}:{} 0x{:X}", file, line, col, hr)]
pub struct HResult {
  pub hr: HRESULT,
  pub file: &'static str,
  pub line: u32,
  pub col: u32,
}

pub type Result<T> = ::std::result::Result<T, HResult>;

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
        Err($crate::hr::HResult {
          hr,
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

#[macro_export]
macro_rules! try_hr {
  ($e:expr) => {{
    let hr = $e;
    if !::winapi::shared::winerror::SUCCEEDED(hr) {
      return Err(HResult {
        hr,
        file: file!(),
        line: line!(),
        col: col!(),
      });
    }
  }};
}
