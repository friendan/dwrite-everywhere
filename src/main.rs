extern crate failure;
extern crate image;
extern crate winapi;
extern crate wio;
#[macro_use]
extern crate failure_derive;
extern crate gamma_lut;

#[macro_use]
pub mod errors;

use image::{RgbImage, RgbaImage};
use std::{
  ffi::OsString,
  mem,
  process::{self, Command},
  ptr,
};
use winapi::{
  shared::minwindef::{FALSE, TRUE},
  um::{
    dcommon, dwrite as dw, dwrite_1 as dw_1, dwrite_2 as dw_2, dwrite_3 as dw_3,
    libloaderapi::LoadLibraryW,
  },
  Interface,
};
use wio::com::ComPtr;
use wio::wide::ToWide;

fn main() {
  unsafe {
    let h = LoadLibraryW(
      OsString::from(
        r"C:\Users\tr\projects\dwrite-everywhere\target\debug\dwrite_everywhere.dll".to_string(),
      ).to_wide_null()
        .as_ptr(),
    );
    assert_ne!(ptr::null_mut(), h);
  };

  //  {
  //    Command::new(r"C:\Program Files\WinFont+\WinFont+64.exe").args(&["-Hook", &process::id().to_string()])
  //      .spawn()
  //      .unwrap()
  //      .wait()
  //      .unwrap();
  //  }

  let (fac, ()): (ComPtr<dw_3::IDWriteFactory3>, ()) = unsafe {
    mem::transmute(
      com_invoke!(
    (dw::DWriteCreateFactory),
    (dw::DWRITE_FACTORY_TYPE_SHARED),
    (&dw_3::IDWriteFactory3::uuidof()),
    (->> p)
    ).unwrap(),
    )
  };
  let (fac_0, ()): (ComPtr<dw::IDWriteFactory>, ()) = unsafe {
    mem::transmute(
      com_invoke!(
    (dw::DWriteCreateFactory),
    (dw::DWRITE_FACTORY_TYPE_SHARED),
    (&dw::IDWriteFactory::uuidof()),
    (->> p)
    ).unwrap(),
    )
  };

  //  let (collection, ()) = com_invoke!(fac.GetSystemFontCollection, FALSE, (->> p), FALSE).unwrap();
  let (collection, ()) = com_invoke!(fac.GetSystemFontCollection, FALSE, (->> p), FALSE).unwrap();

  let (idx, (e, ())) = com_invoke!(
    collection.FindFamilyName,
    (OsString::from("Hasklig".to_string()).to_wide_null().as_ptr()),
    (-> i),
    (-> e)
  ).unwrap();
  if e != TRUE {
    panic!("FontFamilyName:exists")
  }

  let (fam, ()) = com_invoke!(collection.GetFontFamily, idx, (->> p)).unwrap();
  let (font, ()) = com_invoke!(fam.GetFirstMatchingFont, (dw::DWRITE_FONT_WEIGHT_NORMAL), (dw::DWRITE_FONT_STRETCH_NORMAL), (dw::DWRITE_FONT_STYLE_NORMAL), (->> p))
    .unwrap();
  let (face, ()) = com_invoke!(font.CreateFontFace, (->> face)).unwrap();

  let s = b"Unicode"
    .into_iter()
    .map(|&i| i as u32)
    .collect::<Vec<_>>(); // subtable has been obtained, it can be used to retrieve glyph indices for a character code or to implement a number of other useful functions.".into_iter().map(|&i| i as u32).collect::<Vec<_>>();
  let mut ixs = vec![0; s.len()];
  com_invoke!(
    face.GetGlyphIndices,
    (s.as_ptr()),
    (s.len() as u32),
    (ixs.as_mut_ptr())
  ).unwrap();

  let gr = unsafe {
    dw::DWRITE_GLYPH_RUN {
      fontFace: face.as_raw(),
      fontEmSize: 30.0,
      glyphCount: ixs.len() as u32,
      glyphIndices: ixs.as_ptr(),
      ..mem::zeroed()
    }
  };

  //  let (analysis, ()) = com_invoke!(
  //    fac.CreateGlyphRunAnalysis,
  //    (&gr),
  //    (ptr::null()),
  //    (dw_3::DWRITE_RENDERING_MODE1_NATURAL_SYMMETRIC),
  //    (dcommon::DWRITE_MEASURING_MODE_NATURAL),
  //    (dw_2::DWRITE_GRID_FIT_MODE_DISABLED),
  //    (dw_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE),
  //    0.0,
  //    0.0,
  //    (->> p)
  //  ).unwrap();
  let (analysis, ()) = com_invoke!(
    fac_0.CreateGlyphRunAnalysis,
    (&gr),
    1.0,
    (ptr::null()),
    (dw_3::DWRITE_RENDERING_MODE1_ALIASED),
    (dcommon::DWRITE_MEASURING_MODE_NATURAL),
//    (dw_2::DWRITE_GRID_FIT_MODE_DISABLED),
//    (dw_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE),
    0.0,
    0.0,
    (->> p)
  ).unwrap();

  let (bounds, ()) =
    com_invoke!(analysis.GetAlphaTextureBounds, (dw::DWRITE_TEXTURE_ALIASED_1x1), (-> p)).unwrap();

  //  let mut buf1 = RgbImage::new(
  //    (bounds.right - bounds.left) as u32,
  //    (bounds.bottom - bounds.top) as u32,
  //  );
  let mut buf1 =
    vec![0; (bounds.right - bounds.left) as usize * (bounds.bottom - bounds.top) as usize];
  com_invoke!(
    analysis.CreateAlphaTexture,
    (dw::DWRITE_TEXTURE_ALIASED_1x1),
    (&bounds),
    (buf1.as_mut_ptr()),
    (buf1.len() as u32)
  ).unwrap();

  //  let mut buf = RgbaImage::new(buf1.width(), buf1.height());
  let mut buf = RgbaImage::new(
    (bounds.right - bounds.left) as u32,
    (bounds.bottom - bounds.top) as u32,
  );
  //  for (p, p1) in buf.pixels_mut().zip(buf1.pixels()) {
  //    p.data[0] = p1.data[0];
  //    p.data[1] = 255 - p1.data[1];
  //    p.data[2] = 255 - p1.data[2];
  //    p.data[3] = 255;
  //  }
  for (p, &p1) in buf.pixels_mut().zip(buf1.iter()) {
    p.data[0] = p1;
    p.data[1] = p1;
    p.data[2] = p1;
    p.data[3] = 255;
  }
  buf.save("a.bmp").expect("ImageBuffer::save");
}

//////////////////////////////////////////////////////////////////
//extern crate image;
//extern crate kernel32;
//extern crate winapi;
//extern crate wio;
//
//use std::{ffi::OsString, mem, ptr};
//
//extern crate failure;
//#[macro_use]
//extern crate failure_derive;
//
//
//use image::{RgbImage, RgbaImage};
//use kernel32::LoadLibraryW;
//use winapi::{
//  ctypes::c_void,
//  shared::dxgiformat,
//  shared::minwindef::{FALSE, TRUE},
//  shared::winerror::SUCCEEDED,
//  um::{
//    combaseapi, d2d1 as d2d,
//    dcommon::{D2D1_ALPHA_MODE_IGNORE, D2D1_PIXEL_FORMAT, DWRITE_MEASURING_MODE_NATURAL},
//    dwrite as dw, dwrite_1 as dw1, dwrite_2 as dw2, dwrite_3 as dw3,
//    unknwnbase::IUnknown,
//    wincodec,
//  },
//  Interface,
//};
//use wio::{com::ComPtr, wide::ToWide};
//
//fn main() {
//  //  unsafe {
//  //    let h = LoadLibraryW(
//  //      OsString::from(
//  //      r"C:\Users\tr\projects\dwrite-everywhere\target\debug\dwrite_everywhere.dll"
//  //        .to_string()
//  //      )
//  //        .to_wide_null()
//  //        .as_ptr()
//  //    );
//  //
//  //    assert_ne!(ptr::null_mut(), h);
//  //  }
//
//  let (fac, ()): (ComPtr<dw3::IDWriteFactory3>, ()) = unsafe {
//    mem::transmute(
//      com_invoke!(
//    (dw::DWriteCreateFactory),
//    (dw::DWRITE_FACTORY_TYPE_SHARED),
//    (&dw3::IDWriteFactory3::uuidof()),
//    (->> p)
//  ).unwrap(),
//    )
//  };
//
//  let (collection, ()) = com_invoke!(fac.GetSystemFontCollection, FALSE, (->> p), FALSE).unwrap();
//
//  let (idx, (e, ())) = com_invoke!(
//    collection.FindFamilyName,
//    (OsString::from("Hasklig".to_string()).to_wide_null().as_ptr()),
//    (-> i),
//    (-> e)
//  ).unwrap();
//  if e != TRUE {
//    panic!("FontFamilyName:exists")
//  }
//
//  let (fam, ()) = com_invoke!(collection.GetFontFamily, idx, (->> p)).unwrap();
//  let (font, ()) = com_invoke!(fam.GetFirstMatchingFont, (dw::DWRITE_FONT_WEIGHT_NORMAL), (dw::DWRITE_FONT_STRETCH_NORMAL), (dw::DWRITE_FONT_STYLE_NORMAL), (->> p))
//    .unwrap();
//  let (face, ()) = com_invoke!(font.CreateFontFace, (->> face)).unwrap();
//
//  let s = b"Unicode subtable has been obtained, it can be used to retrieve glyph indices for a character code or to implement a number of other useful functions.".into_iter().map(|&i| i as u32).collect::<Vec<_>>();
//  let mut ixs = vec![0; s.len()];
//  com_invoke!(
//    face.GetGlyphIndices,
//    (s.as_ptr()),
//    (s.len() as u32),
//    (ixs.as_mut_ptr())
//  ).unwrap();
//
//  let gr = unsafe {
//    dw::DWRITE_GLYPH_RUN {
//      fontFace: face.as_raw(),
//      fontEmSize: 30.0,
//      glyphCount: ixs.len() as u32,
//      glyphIndices: ixs.as_ptr(),
//      ..mem::zeroed()
//    }
//  };
//
//  let (analysis, ()) = com_invoke!(
//    fac.CreateGlyphRunAnalysis,
//    (&gr),
//    (ptr::null()),
//    (dw3::DWRITE_RENDERING_MODE1_NATURAL_SYMMETRIC),
//    DWRITE_MEASURING_MODE_NATURAL,
//    (dw2::DWRITE_GRID_FIT_MODE_DISABLED),
//    (dw1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE),
//    0.0,
//    0.0,
//    (->> p)
//  ).unwrap();
//
//  let (bounds, ()) = com_invoke!(analysis.GetAlphaTextureBounds, (dw::DWRITE_TEXTURE_CLEARTYPE_3x1), (-> p))
//    .unwrap();
//  println!("{:?}", bounds);
//
//  let mut buf1 = RgbImage::new(
//    (bounds.right - bounds.left) as u32,
//    (bounds.bottom - bounds.top) as u32,
//  );
//  com_invoke!(
//    analysis.CreateAlphaTexture,
//    (dw::DWRITE_TEXTURE_CLEARTYPE_3x1),
//    (&bounds),
//    (buf1.as_mut_ptr()),
//    (buf1.len() as u32)
//  ).unwrap();
//
//  let mut buf = RgbaImage::new(buf1.width(), buf1.height());
//  for (p, p1) in buf.pixels_mut().zip(buf1.pixels()) {
//    p.data[0] = p1.data[2];
//    p.data[1] = p1.data[1];
//    p.data[2] = p1.data[0];
//    p.data[3] = 255;
//  }
//
//  let d2d_fac = unsafe {
//    ComPtr::from_raw(
//      com_invoke!(
//    (d2d::D2D1CreateFactory),
//    (d2d::D2D1_FACTORY_TYPE_SINGLE_THREADED),
//    (&d2d::ID2D1Factory::uuidof()),
//    (ptr::null()),
//    (-> p)
//  ).unwrap()
//        .0 as *mut d2d::ID2D1Factory,
//    )
//  };
//
//  com_invoke!(
//    (combaseapi::CoInitializeEx),
//    (ptr::null_mut()),
//    (combaseapi::COINITBASE_MULTITHREADED)
//  ).unwrap();
//
//  let wic_fac = unsafe {
//    ComPtr::from_raw(
//      com_invoke!(
//    (combaseapi::CoCreateInstance),
//    (&wincodec::CLSID_WICImagingFactory),
//    (ptr::null_mut()),
//    (combaseapi::CLSCTX_INPROC),
//    (&wincodec::IWICImagingFactory::uuidof()),
//    (-> p)
//  ).unwrap()
//        .0 as *mut wincodec::IWICImagingFactory,
//    )
//  };
//
//  let (wic_bm, _) = com_invoke!(
//    wic_fac.CreateBitmapFromMemory,
//    (buf.width()),
//    (buf.height()),
//    (&wincodec::GUID_WICPixelFormat32bppBGR),
//    (4 * buf.width()),
//    (buf.len() as u32),
//    (buf.as_mut_ptr()),
//    (->> p)
//  ).unwrap();
//
//  let (rt, _) = unsafe {
//    let props = d2d::D2D1_RENDER_TARGET_PROPERTIES {
//      pixelFormat: D2D1_PIXEL_FORMAT {
//        format: dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
//        alphaMode: D2D1_ALPHA_MODE_IGNORE,
//      },
//      ..mem::zeroed()
//    };
//    com_invoke!(
//      d2d_fac.CreateWicBitmapRenderTarget,
//      (wic_bm.as_raw()),
//      (&props),
//      (->> p)
//    ).unwrap()
//  };
//
//  let (path_geo, ()) = com_invoke!(d2d_fac.CreatePathGeometry, (->> p)).unwrap();
//
//  let (sink, ()) = com_invoke!(path_geo.Open, (->> p)).unwrap();
//  com_invoke!(
//    face.GetGlyphRunOutline,
//    (gr.fontEmSize),
//    (ixs.as_ptr()),
//    (ptr::null()),
//    (ptr::null()),
//    (ixs.len() as u32),
//    FALSE,
//    FALSE,
//    (sink.as_raw() as *mut d2d::ID2D1SimplifiedGeometrySink)
//  ).unwrap();
//
//  unsafe {
//    sink.Close();
//  }
//
//  let (transformed_path, ()) = {
//    let mat = d2d::D2D1_MATRIX_3X2_F {
//      //      matrix: [[1.0, 0.0], [0.0, 1.0], [-2.0, 22.0]],
//      matrix: [
//        [1.0, 0.0],
//        [0.0, 1.0],
//        [-bounds.left as f32, -bounds.top as f32],
//      ],
//    };
//    com_invoke!(d2d_fac.CreateTransformedGeometry, (path_geo.as_raw() as *mut d2d::ID2D1Geometry), (&mat), (->> p))
//      .unwrap()
//  };
//
//  let (brush, ()) = {
//    let color = d2d::D2D1_COLOR_F {
//      r: 1.0,
//      g: 1.0,
//      b: 1.0,
//      a: 1.0,
//    };
//    com_invoke!(rt.CreateSolidColorBrush, (&color), (ptr::null()), (->> p)).unwrap()
//  };
//
//  unsafe {
//    rt.BeginDraw();
//    rt.DrawGeometry(
//      transformed_path.as_raw() as *mut d2d::ID2D1Geometry,
//      brush.as_raw() as *mut d2d::ID2D1Brush,
//      0.6,
//      ptr::null_mut(),
//    );
//    com_invoke!(rt.EndDraw, (ptr::null_mut()), (ptr::null_mut())).unwrap();
//  }
//
//  com_invoke!(
//    wic_bm.CopyPixels,
//    (ptr::null()),
//    (4 * buf.width()),
//    (buf.len() as u32),
//    (buf.as_mut_ptr())
//  ).unwrap();
//
//  for p in buf.pixels_mut() {
//    p.data.swap(0, 2);
//  }
//
//  buf.save("a.bmp").expect("ImageBuffer::save");
//}
