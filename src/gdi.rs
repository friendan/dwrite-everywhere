use std::{
  mem,
  ptr::{null, null_mut},
  slice,
};

use winapi::{
  ctypes::c_void,
  shared::{dxgiformat, windef},
  um::{d2d1, dcommon, dwrite, wingdi},
};

use errors::HResult;

lazy_static! {
//  static ref EN_US: Vec<u16> = OsString::from("en-us".to_string()).to_wide_null();
//
//  static ref HFONT_TO_TF: RwLock<HashMap<UnsafeSendSync<HFONT>, ComPtr<dwrite::IDWriteTextFormat>>> =
//    RwLock::new(HashMap::new());
}

//pub fn create_font_indirect_ex_w(
//  dw_fac: *mut dwrite::IDWriteFactory,
//  sys_collection: *mut dwrite::IDWriteFontCollection,
//  lfex: *const wingdi::ENUMLOGFONTEXDVW,
//) -> HResult<windef::HFONT> {
//  let h_font = wingdi::CreateFontIndirectExW(lfex);
//  if h_font == null_mut() {
//    return Ok(h_font);
//  }
//
//  let dw_fac = unsafe { &*dw_fac };
//  let wingdi::ENUMLOGFONTEXDVW {
//    elfEnumLogfontEx:
//      wingdi::ENUMLOGFONTEXW {
//        elfLogFont:
//          wingdi::LOGFONTW {
//            lfFacename: family_name,
//            lfWeight: weight,
//            lfHeight: size,
//            lfItalic: is_italic,
//            ..
//          },
//        ..
//      },
//    ..
//  } = unsafe { &*lfex };
//  let (text_format, ()) = com_invoke!(
//    (dw_fac.CreateTextFormat),
//    (&*family_name),
//    sys_collection,
//    weight,
//    (if is_italic { dwrite::DWRITE_FONT_STYLE_ITALIC } else { dwrite::DWRITE_FONT_STYLE }),
//    (dwrite::DWRITE_FONT_STRETCH_NORMAL), // TODO
//    (size as f32),
//    EN_US.as_ptr(), // TODO
//    (->> p),
//  )?;
//
//  HFONT_TO_TF
//    .write()
//    .unwrap()
//    .insert(unsafe { UnsafeSendSync::new(h_font) }, text_format);
//  Ok(hfont)
//}

pub fn ext_text_out_w(
  d2d_fac: *mut d2d1::ID2D1Factory,
  dw_gdi: *mut dwrite::IDWriteGdiInterop,
  hdc: windef::HDC,
  x: i32,
  y: i32,
  options: u32,
  rect: *const windef::RECT,
  s: *const u16,
  c: u32,
  dxs: *const i32,
) -> HResult<()> {
  let d2d_fac = unsafe { &*d2d_fac };
  let dw_gdi = unsafe { &*dw_gdi };

  let (font_face, ()) = com_invoke!(
    dw_gdi.CreateFontFaceFromHdc,
    hdc,
    (->> p)
  )?;

  let mut glyphs_buf = vec![];
  let glyphs = if (options & wingdi::ETO_GLYPH_INDEX) != wingdi::ETO_GLYPH_INDEX {
    let codepoints: Vec<_> = String::from_utf16(unsafe { slice::from_raw_parts(s, c as usize) })
      .map_err(|_| annotate_error!(0))?
      .chars()
      .map(|c| c as u32)
      .collect();
    glyphs_buf.resize(codepoints.len(), 0);
    com_invoke!(
      font_face.GetGlyphIndices,
      (codepoints.as_ptr()),
      (codepoints.len() as u32),
      (glyphs_buf.as_mut_ptr())
    )?;
    &glyphs_buf
  } else {
    unsafe { slice::from_raw_parts(s, c as usize) }
  };

  // font, bg color, text color
  let (rt, ()) = {
    let props = d2d1::D2D1_RENDER_TARGET_PROPERTIES {
      _type: d2d1::D2D1_RENDER_TARGET_TYPE_DEFAULT,
      pixelFormat: dcommon::D2D1_PIXEL_FORMAT {
        format: dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        alphaMode: dcommon::D2D1_ALPHA_MODE_IGNORE,
      },
      dpiX: 0.0,
      dpiY: 0.0,
      usage: d2d1::D2D1_RENDER_TARGET_USAGE_NONE,
      minLevel: d2d1::D2D1_FEATURE_LEVEL_DEFAULT,
    };
    com_invoke!(
      d2d_fac.CreateDCRenderTarget,
      (&props),
      (->> p)
    )?
  };

  //  let h_font = unsafe { wingdi::GetCurrentObject(hdc, wingdi::OBJ_FONT) };
  //  let text_layout = HFONT_TO_TF
  //    .read()
  //    .get(unsafe { &UnsafeSendSync::new(h_font) })
  //    .ok_or(annotate_error!(0))?;
  //  let lf = unsafe {
  //    let mut lf = mem::uninitialized();
  //    let r = wingdi::GetObjectW(
  //      h_font,
  //      mem::size_of::<wingdi::LOGFONTW>(),
  //      &mut lf
  //    );
  //    if r == 0 {
  //      return Err(annotate_error!(0))
  //    }
  //  };

  let (fg_brush, ()) = {
    let fg = unsafe { wingdi::GetTextColor(hdc) };
    com_invoke!(
      rt.CreateSolidColorBrush,
      (&d2d1::D2D1_COLOR_F {
        r: (wingdi::GetRValue(fg) as f32) / 255.0,
        g: (wingdi::GetGValue(fg) as f32) / 255.0,
        b: (wingdi::GetBValue(fg) as f32) / 255.0,
        a: 1.0,
      }),
      (null()),
      (->> p)
    )?
  };

  let h_font = unsafe { wingdi::GetCurrentObject(hdc, wingdi::OBJ_FONT) };
  let lf = unsafe {
    let mut lf: wingdi::LOGFONTW = mem::uninitialized();
    let r = wingdi::GetObjectW(
      h_font,
      mem::size_of::<wingdi::LOGFONTW>() as i32,
      &mut lf as *mut wingdi::LOGFONTW as *mut c_void,
    );
    if r == 0 {
      return Err(annotate_error!(0));
    }
    lf
  };
  let glyph_run = dwrite::DWRITE_GLYPH_RUN {
    fontFace: font_face.as_raw(),
    fontEmSize: lf.lfHeight as f32,
    glyphCount: glyphs.len() as u32,
    glyphIndices: glyphs.as_ptr(),
    glyphAdvances: null(),
    glyphOffsets: null(),
    isSideways: 0,
    bidiLevel: 0,
  };

  unsafe { rt.BeginDraw() };
  unsafe {
    rt.DrawGlyphRun(
      d2d1::D2D1_POINT_2F {
        x: x as f32,
        y: y as f32,
      },
      &glyph_run,
      fg_brush.as_raw() as *mut d2d1::ID2D1Brush,
      dcommon::DWRITE_MEASURING_MODE_NATURAL,
    )
  };
  com_invoke!(rt.EndDraw, (null_mut()), (null_mut()))?;

  Ok(())
}
