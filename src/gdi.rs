use std::{
  mem,
  ptr::{null, null_mut},
  slice,
};

use winapi::{
  ctypes::c_void,
  shared::{dxgiformat, windef},
  um::{d2d1, dcommon, dwrite, dwrite_1, dwrite_2, dwrite_3, wingdi},
};

use errors::HResult;
use fns;

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
//    dw_fac.CreateTextFormat,
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

pub fn text_out_w(
  _: fns::TextOutW,
  hdc: windef::HDC,
  x: i32,
  y: i32,
  s: *const u16,
  c: i32,
) -> i32 {
  unsafe { wingdi::ExtTextOutW(hdc, x, y, 0, null(), s, c as u32, null()) }
}

pub fn ext_text_out_w(
  dw_fac_3: *mut dwrite_3::IDWriteFactory3,
  d2d_fac: *mut d2d1::ID2D1Factory,
  dw_gdi: *mut dwrite::IDWriteGdiInterop,
  create_glyph_run_analysis: fns::CreateGlyphRunAnalysis3,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
  tramp: fns::ExtTextOutW,
  hdc: windef::HDC,
  x: i32,
  y: i32,
  options: u32,
  rect: *const windef::RECT,
  s: *const u16,
  c: u32,
  dxs: *const i32,
) -> i32 {
  match ext_text_out_w_impl(
    dw_fac_3,
    d2d_fac,
    dw_gdi,
    create_glyph_run_analysis,
    get_alpha_texture_bounds,
    tramp,
    hdc,
    x,
    y,
    options,
    rect,
    s,
    c,
    dxs,
  ) {
    Ok(()) => 1,
    Err(e) => {
      println!("ExtTextOutW: {:?}", e);
      unsafe { tramp(hdc, x, y, options, rect, s, c, dxs) }
    }
  }
}

fn ext_text_out_w_impl(
  dw_fac_3: *mut dwrite_3::IDWriteFactory3,
  d2d_fac: *mut d2d1::ID2D1Factory,
  dw_gdi: *mut dwrite::IDWriteGdiInterop,
  create_glyph_run_analysis: fns::CreateGlyphRunAnalysis3,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
  tramp: fns::ExtTextOutW,
  hdc: windef::HDC,
  mut x: i32,
  mut y: i32,
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
    fontEmSize: lf.lfHeight.abs() as f32,
    glyphCount: glyphs.len() as u32,
    glyphIndices: glyphs.as_ptr(),
    glyphAdvances: null(),
    glyphOffsets: null(),
    isSideways: 0,
    bidiLevel: 0,
  };

  let align = unsafe { wingdi::GetTextAlign(hdc) };
  if (align & wingdi::TA_UPDATECP) != 0 {
    let pt = unsafe {
      let mut pt = mem::uninitialized();
      wingdi::GetCurrentPositionEx(hdc, &mut pt);
      pt
    };
    x = pt.x;
    y = pt.y;
  }

  let is_cleartype = (lf.lfQuality as u32) == wingdi::CLEARTYPE_NATURAL_QUALITY
    || (lf.lfQuality as u32) == wingdi::CLEARTYPE_QUALITY;

  let (gla, ()) = com_invoke!(
    (create_glyph_run_analysis),
    dw_fac_3,
    (&glyph_run),
    (null()),
    (dwrite::DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC),
    (dcommon::DWRITE_MEASURING_MODE_NATURAL),
    (dwrite_2::DWRITE_GRID_FIT_MODE_DISABLED),
    (if is_cleartype {
      dwrite_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE
    } else {
      dwrite_1::DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE
    }),
    (x as f32),
    (y as f32),
    (->> p)
  )?;

  let (bounds, ()) = com_invoke!(
    (get_alpha_texture_bounds),
    (gla.as_raw()),
    (if is_cleartype
      { dwrite::DWRITE_TEXTURE_CLEARTYPE_3x1  }
     else
      { dwrite::DWRITE_TEXTURE_ALIASED_1x1 }),
    (-> p)
  )?;
  let (baseline_x, baseline_y, mut render_rect) = render_bounds(hdc, x, y, align, bounds);
  if (options & wingdi::ETO_CLIPPED) != 0 {
//    println!("clipped ({} {}) {:?} {:?}", x, y, unsafe { &*rect }, render_rect);
    render_rect = translate_rc(unsafe { &*rect }, );
  }

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

  com_invoke!(rt.BindDC, hdc, (&render_rect))?;
  unsafe { rt.BeginDraw() };

  if options & wingdi::ETO_OPAQUE != 0 {
    let bg = unsafe { wingdi::GetBkColor(hdc) };
    let (bg_brush, ()) = com_invoke!(
      rt.CreateSolidColorBrush,
      (&color_ref_to_d2d_color_f(bg)),
      (null()),
      (->> p)
    )?;
    unsafe {
      rt.FillRectangle(&rect_to_d2d1_rect_f(&render_rect), bg_brush.as_raw() as *mut d2d1::ID2D1Brush);
    }
  }

  let (fg_brush, ()) = {
    let fg = unsafe { wingdi::GetTextColor(hdc) };
    com_invoke!(
      rt.CreateSolidColorBrush,
      (&color_ref_to_d2d_color_f(fg)),
      (null()),
      (->> p)
    )?
  };
  unsafe {
    rt.DrawGlyphRun(
      d2d1::D2D1_POINT_2F {
        x: baseline_x as f32,
        y: baseline_y as f32,
      },
      &glyph_run,
      fg_brush.as_raw() as *mut d2d1::ID2D1Brush,
      dcommon::DWRITE_MEASURING_MODE_NATURAL,
    );
  };
  com_invoke!(rt.EndDraw, (null_mut()), (null_mut()))?;

  Ok(())
}

fn get_text_extent_point_32_impl(
  hdc: windef::HDC,
  s: *const u16,
  
) -> u32 {

}

#[derive(Debug)]
struct DebugTextMetric {
  tmHeight: i32,
  tmAscent: i32,
  tmDescent: i32,
  tmInternalLeading: i32,
  tmExternalLeading: i32,
  tmAveCharWidth: i32,
  tmMaxCharWidth: i32,
  tmWeight: i32,
  tmOverhang: i32,
  tmDigitizedAspectX: i32,
  tmDigitizedAspectY: i32,
  tmFirstChar: u16,
  tmLastChar: u16,
  tmDefaultChar: u16,
  tmBreakChar: u16,
  tmItalic: u8,
  tmUnderlined: u8,
  tmStruckOut: u8,
  tmPitchAndFamily: u8,
  tmCharSet: u8,
}

impl From<wingdi::TEXTMETRICW> for DebugTextMetric {
  fn from(w: wingdi::TEXTMETRICW) -> Self {
    Self {
      tmHeight: w.tmHeight,
      tmAscent: w.tmAscent,
      tmDescent: w.tmDescent,
      tmInternalLeading: w.tmInternalLeading,
      tmExternalLeading: w.tmExternalLeading,
      tmAveCharWidth: w.tmAveCharWidth,
      tmMaxCharWidth: w.tmMaxCharWidth,
      tmWeight: w.tmWeight,
      tmOverhang: w.tmOverhang,
      tmDigitizedAspectX: w.tmDigitizedAspectX,
      tmDigitizedAspectY: w.tmDigitizedAspectY,
      tmFirstChar: w.tmFirstChar,
      tmLastChar: w.tmLastChar,
      tmDefaultChar: w.tmDefaultChar,
      tmBreakChar: w.tmBreakChar,
      tmItalic: w.tmItalic,
      tmUnderlined: w.tmUnderlined,
      tmStruckOut: w.tmStruckOut,
      tmPitchAndFamily: w.tmPitchAndFamily,
      tmCharSet: w.tmCharSet,
    }
  }
}

fn render_bounds(hdc: windef::HDC, x: i32, y: i32, align: u32, dw_bounds: windef::RECT) -> (i32, i32, windef::RECT) {
  let w = dw_bounds.right - dw_bounds.left;
  let h = dw_bounds.bottom - dw_bounds.top;
  let (dx, dy) = {
    if (align & wingdi::TA_BASELINE) != 0 {
      if (align & wingdi::TA_RIGHT) != 0 {
        (-w + (x - dw_bounds.left), 0)
      } else if (align & wingdi::TA_CENTER) != 0 {
        (-w / 2 + (x - dw_bounds.left), 0)
      } else {
        (x - dw_bounds.left, 0)
      }
    } else if (align & wingdi::TA_BOTTOM) != 0 {
      if (align & wingdi::TA_RIGHT) != 0 {
        (-w + (x - dw_bounds.left), -h + (dw_bounds.bottom - y))
      } else if (align & wingdi::TA_CENTER) != 0 {
        (-w / 2 + (x - dw_bounds.left), -h + (dw_bounds.bottom - y))
      } else {
        (x - dw_bounds.left, dw_bounds.bottom - y)
      }
    } else {
      if (align & wingdi::TA_RIGHT) != 0 {
        (w - (x - dw_bounds.left), h - (dw_bounds.bottom - y))
      } else if (align & wingdi::TA_CENTER) != 0 {
        (-w / 2 + (x - dw_bounds.left), h - (dw_bounds.bottom - y))
      } else {
        // Left-top
        (x - dw_bounds.left, y - dw_bounds.top)
      }
    }
  };

  let tm = unsafe {
    let mut tm: wingdi::TEXTMETRICW = mem::uninitialized();
    wingdi::GetTextMetricsW(hdc, &mut tm);
    tm
  };

  let bounds = translate_rc(&dw_bounds, dx, dy);
  let baseline_x = x - dw_bounds.left;
  let baseline_y = y - dw_bounds.top;

  if h != tm.tmHeight {
    (baseline_x, baseline_y + tm.tmInternalLeading, windef::RECT {
      top: bounds.top,
      bottom: bounds.bottom + tm.tmInternalLeading + tm.tmDescent,
      ..bounds
    })
  } else {
  (baseline_x, baseline_y, bounds)
  }
}

fn color_ref_to_d2d_color_f(cr: windef::COLORREF) -> d2d1::D2D1_COLOR_F {
  d2d1::D2D1_COLOR_F {
      r: (wingdi::GetRValue(cr) as f32) / 255.0,
      g: (wingdi::GetGValue(cr) as f32) / 255.0,
      b: (wingdi::GetBValue(cr) as f32) / 255.0,
      a: 1.0,
  }
}

fn rect_to_d2d1_rect_f(rc: &windef::RECT) -> d2d1::D2D1_RECT_F {
  d2d1::D2D1_RECT_F {
    left: rc.left as f32,
    top: rc.top as f32,
    right: rc.right as f32,
    bottom: rc.bottom as f32,
  }
}

fn translate_rc(rc: &windef::RECT, dx: i32, dy: i32) -> windef::RECT {
  windef::RECT {
    left: rc.left + dx,
    top: rc.top + dy,
    right: rc.right + dx,
    bottom: rc.bottom + dy,
  }
}
