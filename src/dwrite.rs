use super::d2d1_helper::matrix_3x2_f;
use super::errors;
use super::fns;
use super::util::UnsafeSendSync;
use std::{collections::HashMap, mem, ptr, slice, sync::RwLock};
use winapi::{
  shared::{
    dxgiformat,
    minwindef::{BOOL, FLOAT, ULONG},
    windef::RECT,
    winerror::SUCCEEDED,
  },
  um::{
    d2d1::{self, D2D1_MATRIX_3X2_F},
    dcommon, dwrite as dw, dwrite_1 as dw_1, dwrite_2 as dw_2, dwrite_3 as dw_3,
    unknwnbase::IUnknown,
    wincodec,
    winnt::HRESULT,
  },
};
use wio::com::ComPtr;

lazy_static! {
  static ref GRA_CTXS: RwLock<HashMap<UnsafeSendSync<*const dw::IDWriteGlyphRunAnalysis>, GlyphRunData>> =
    RwLock::new(HashMap::new());
}

struct GlyphRunData {
  bgra_bm: ComPtr<wincodec::IWICBitmap>,
  bounds: RECT,
}

unsafe impl Send for GlyphRunData {}

unsafe impl Sync for GlyphRunData {}

pub fn detour_create_glyph_run_analysis(
  dw_fac_3: *mut dw_3::IDWriteFactory3,
  wic_fac: *mut wincodec::IWICImagingFactory,
  d2d_fac: *mut d2d1::ID2D1Factory,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
  create_glyph_run_analysis_3: fns::CreateGlyphRunAnalysis3,
  this: *mut dw::IDWriteFactory,
  glyph_run_1: *const dw::DWRITE_GLYPH_RUN,
  pixels_per_dip: FLOAT,
  transform: *const dw::DWRITE_MATRIX,
  rendering_mode: dw::DWRITE_RENDERING_MODE,
  measuring_mode: dcommon::DWRITE_MEASURING_MODE,
  mut baseline_origin_x: FLOAT,
  mut baseline_origin_y: FLOAT,
  glyph_run_analysis: *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT {
  let glyph_run = dw::DWRITE_GLYPH_RUN {
    fontEmSize: (unsafe { *glyph_run_1 }).fontEmSize * pixels_per_dip,
    ..unsafe { *glyph_run_1 }
  };
  baseline_origin_x *= pixels_per_dip;
  baseline_origin_y *= pixels_per_dip;

  let hr = unsafe {
    create_glyph_run_analysis_3(
      dw_fac_3,
      &glyph_run,
      transform,
      rendering_mode,
      measuring_mode,
      dw_2::DWRITE_GRID_FIT_MODE_DISABLED,
      if rendering_mode == dw::DWRITE_RENDERING_MODE_ALIASED {
        dw_1::DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE
      } else {
        dw_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE
      },
      baseline_origin_x,
      baseline_origin_y,
      glyph_run_analysis,
    )
  };

  if SUCCEEDED(hr) {
    store_glyph_run(
      this,
      wic_fac,
      d2d_fac,
      unsafe { *glyph_run_analysis },
      &glyph_run,
      transform,
      if rendering_mode == dw::DWRITE_RENDERING_MODE_ALIASED {
        dw::DWRITE_TEXTURE_ALIASED_1x1
      } else {
        dw::DWRITE_TEXTURE_CLEARTYPE_3x1
      },
      baseline_origin_x,
      baseline_origin_y,
      get_alpha_texture_bounds,
    )
  } else {
    hr
  }
}

pub fn detour_create_glyph_run_analysis_2(
  wic_fac: *mut wincodec::IWICImagingFactory,
  d2d_fac: *mut d2d1::ID2D1Factory,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
  tramp: fns::CreateGlyphRunAnalysis2,
  this: *mut dw_2::IDWriteFactory2,
  glyph_run: *const dw::DWRITE_GLYPH_RUN,
  transform: *const dw::DWRITE_MATRIX,
  rendering_mode: dw::DWRITE_RENDERING_MODE,
  measuring_mode: dcommon::DWRITE_MEASURING_MODE,
  _grid_fit_mode: dw_2::DWRITE_GRID_FIT_MODE,
  antialias_mode: dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
  baseline_origin_x: FLOAT,
  baseline_origin_y: FLOAT,
  glyph_run_analysis: *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT {
  let hr = unsafe {
    tramp(
      this,
      glyph_run,
      transform,
      rendering_mode,
      measuring_mode,
      dw_2::DWRITE_GRID_FIT_MODE_DISABLED,
      antialias_mode,
      baseline_origin_x,
      baseline_origin_y,
      glyph_run_analysis,
    )
  };
  if SUCCEEDED(hr) {
    store_glyph_run(
      this as *mut dw::IDWriteFactory,
      wic_fac,
      d2d_fac,
      unsafe { *glyph_run_analysis },
      glyph_run,
      transform,
      if antialias_mode == dw_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE {
        dw::DWRITE_TEXTURE_CLEARTYPE_3x1
      } else {
        dw::DWRITE_TEXTURE_ALIASED_1x1
      },
      baseline_origin_x,
      baseline_origin_y,
      get_alpha_texture_bounds,
    )
  } else {
    hr
  }
}

pub fn detour_create_glyph_run_analysis_3(
  wic_fac: *mut wincodec::IWICImagingFactory,
  d2d_fac: *mut d2d1::ID2D1Factory,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
  tramp: fns::CreateGlyphRunAnalysis3,
  this: *mut dw_3::IDWriteFactory3,
  glyph_run: *const dw::DWRITE_GLYPH_RUN,
  transform: *const dw::DWRITE_MATRIX,
  rendering_mode: dw_3::DWRITE_RENDERING_MODE1,
  measuring_mode: dcommon::DWRITE_MEASURING_MODE,
  grid_fit_mode: dw_2::DWRITE_GRID_FIT_MODE,
  antialias_mode: dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
  baseline_origin_x: FLOAT,
  baseline_origin_y: FLOAT,
  glyph_run_analysis: *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT {
  let hr = unsafe {
    tramp(
      this,
      glyph_run,
      transform,
      rendering_mode,
      measuring_mode,
      grid_fit_mode,
      antialias_mode,
      baseline_origin_x,
      baseline_origin_y,
      glyph_run_analysis,
    )
  };
  if SUCCEEDED(hr) {
    store_glyph_run(
      this as *mut dw::IDWriteFactory,
      wic_fac,
      d2d_fac,
      unsafe { *glyph_run_analysis },
      glyph_run,
      transform,
      if antialias_mode == dw_1::DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE {
        dw::DWRITE_TEXTURE_CLEARTYPE_3x1
      } else {
        dw::DWRITE_TEXTURE_ALIASED_1x1
      },
      baseline_origin_x,
      baseline_origin_y,
      get_alpha_texture_bounds,
    )
  } else {
    hr
  }
}

fn store_glyph_run(
  dw_fac: *mut dw::IDWriteFactory,
  wic_fac: *mut wincodec::IWICImagingFactory,
  d2d_fac: *mut d2d1::ID2D1Factory,
  gla: *mut dw::IDWriteGlyphRunAnalysis,
  glyph_run: *const dw::DWRITE_GLYPH_RUN,
  transform: *const dw::DWRITE_MATRIX,
  texture_type: dw::DWRITE_TEXTURE_TYPE,
  baseline_x: FLOAT,
  baseline_y: FLOAT,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
) -> HRESULT {
  let id = matrix_3x2_f::id();
  let trans = if transform == ptr::null() {
    id
  } else {
    unsafe { matrix_3x2_f::from_dwrite_matrix(&*transform) }
  };
  match embolden2(
    unsafe { &mut *dw_fac },
    unsafe { &mut *wic_fac },
    unsafe { &mut *d2d_fac },
    gla,
    unsafe { &*glyph_run },
    trans,
    texture_type,
    baseline_x,
    baseline_y,
    get_alpha_texture_bounds,
  ) {
    Ok(data) => {
      (*GRA_CTXS)
        .write()
        .unwrap()
        .insert(unsafe { UnsafeSendSync::new(gla) }, data);
      0
    }
    Err(e) => e.error,
  }
}

pub fn detour_get_alpha_texture_bounds(
  tramp: unsafe extern "system" fn(
    *mut dw::IDWriteGlyphRunAnalysis,
    dw::DWRITE_TEXTURE_TYPE,
    *mut RECT,
  ) -> HRESULT,
  this: *mut dw::IDWriteGlyphRunAnalysis,
  texture_type: dw::DWRITE_TEXTURE_TYPE,
  rc: *mut RECT,
) -> HRESULT {
  if let Some(data) = (*GRA_CTXS)
    .read()
    .unwrap()
    .get(unsafe { &UnsafeSendSync::new(this) })
  {
    unsafe {
      *rc = data.bounds;
    }
    0
  } else {
    unsafe { tramp(this, texture_type, rc) }
  }
}

fn embolden2(
  dw_fac: &mut dw::IDWriteFactory,
  wic_fac: &mut wincodec::IWICImagingFactory,
  d2d_fac: &mut d2d1::ID2D1Factory,
  gla: *mut dw::IDWriteGlyphRunAnalysis,
  glyph_run: &dw::DWRITE_GLYPH_RUN,
  transform: D2D1_MATRIX_3X2_F,
  texture_type: dw::DWRITE_TEXTURE_TYPE,
  baseline_x: FLOAT,
  baseline_y: FLOAT,
  get_alpha_texture_bounds: fns::GetAlphaTextureBounds,
) -> errors::HResult<GlyphRunData> {
  let (mut bounds, ()) = com_invoke!((get_alpha_texture_bounds), gla, texture_type, (-> p))?;
  // Leave space for outline.
  bounds.left -= 1;
  bounds.top -= 1;
  bounds.right += 1;
  bounds.bottom += 1;

  let width = bounds.right - bounds.left;
  let height = bounds.bottom - bounds.top;

  let (bgra_bm, ()) = com_invoke!(
    wic_fac.CreateBitmap,
    (width as u32),
    (height as u32),
    (&wincodec::GUID_WICPixelFormat32bppBGR),
    (wincodec::WICBitmapCacheOnDemand),
    (->> p)
  )?;

  let (transformed_outline, ()) = {
    let (outline, ()) = com_invoke!(d2d_fac.CreatePathGeometry, (->> p))?;
    let (sink, ()) = com_invoke!(outline.Open, (->> p))?;
    com_invoke!(
      ((*glyph_run.fontFace).GetGlyphRunOutline),
      (glyph_run.fontEmSize),
      (glyph_run.glyphIndices),
      (glyph_run.glyphAdvances),
      (glyph_run.glyphOffsets),
      (glyph_run.glyphCount),
      (glyph_run.isSideways),
      ((glyph_run.bidiLevel % 2) as BOOL),
      (sink.as_raw() as *mut d2d1::ID2D1SimplifiedGeometrySink)
    )?;
    unsafe {
      sink.Close();
    }

    com_invoke!(
        d2d_fac.CreateTransformedGeometry,
        (outline.as_raw() as *mut d2d1::ID2D1Geometry),
        (&matrix_3x2_f::mul(&transform, &matrix_3x2_f::translate(baseline_x + 1.0 - (bounds.left as f32), baseline_y + 1.0 - (bounds.top as f32)))),
        (->> p)
      )?
  };

  let (rt, ()) = unsafe {
    let props = d2d1::D2D1_RENDER_TARGET_PROPERTIES {
      pixelFormat: dcommon::D2D1_PIXEL_FORMAT {
        format: dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        alphaMode: dcommon::D2D1_ALPHA_MODE_IGNORE,
      },
      ..mem::zeroed()
    };
    com_invoke!(
        d2d_fac.CreateWicBitmapRenderTarget,
        (bgra_bm.as_raw()),
        (&props),
        (->> p)
      )?
  };

  let (outline_brush, ()) = {
    let color = d2d1::D2D1_COLOR_F {
      r: 0.6,
      g: 0.6,
      b: 0.6,
      a: 1.0,
    };
    com_invoke!(rt.CreateSolidColorBrush, (&color), (ptr::null()), (->> p))?
  };

  let (text_brush, ()) = {
    let color = d2d1::D2D1_COLOR_F {
      r: 0.9,
      g: 0.9,
      b: 0.9,
      a: 1.0,
    };
    com_invoke!(rt.CreateSolidColorBrush, (&color), (ptr::null()), (->> p))?
  };

  unsafe {
    let (rendering_params, _) = com_invoke!(
      dw_fac.CreateCustomRenderingParams,
      1.0,
      0.0,
      (if texture_type == dw::DWRITE_TEXTURE_CLEARTYPE_3x1 { 1.0 } else { 0.0 }),
      (dw::DWRITE_PIXEL_GEOMETRY_RGB),
      (dw::DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC),
      (->> p)
    )?;
    rt.SetTextRenderingParams(rendering_params.as_raw());
    rt.SetTextAntialiasMode(if texture_type == dw::DWRITE_TEXTURE_CLEARTYPE_3x1 {
      d2d1::D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE
    } else {
      d2d1::D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE
    });
  }

  unsafe {
    rt.BeginDraw();
    rt.DrawGeometry(
      transformed_outline.as_raw() as *mut d2d1::ID2D1Geometry,
      outline_brush.as_raw() as *mut d2d1::ID2D1Brush,
      0.6,
      ptr::null_mut(),
    );
    rt.SetTransform(&matrix_3x2_f::mul(
      &transform,
      &matrix_3x2_f::translate(
        baseline_x + 1.0 - (bounds.left as f32),
        baseline_y + 1.0 - (bounds.top as f32),
      ),
    ));
    rt.DrawGlyphRun(
      d2d1::D2D1_POINT_2F { x: 0.0, y: 0.0 },
      glyph_run,
      text_brush.as_raw() as *mut d2d1::ID2D1Brush,
      dcommon::DWRITE_MEASURING_MODE_NATURAL,
    );
    com_invoke!(rt.EndDraw, (ptr::null_mut()), (ptr::null_mut()))?
  };

  Ok(GlyphRunData { bgra_bm, bounds })
}

pub fn detour_create_alpha_texture(
  tramp: fns::CreateAlphaTexture,
  this: *mut dw::IDWriteGlyphRunAnalysis,
  texture_type: dw::DWRITE_TEXTURE_TYPE,
  texture_bounds: *const RECT,
  alpha_values: *mut u8,
  buffer_size: u32,
) -> HRESULT {
  if let Some(data) = (*GRA_CTXS)
    .read()
    .unwrap()
    .get(unsafe { &UnsafeSendSync::new(this) })
  {
    copy_texture(data, texture_type, unsafe { &*texture_bounds }, unsafe {
      slice::from_raw_parts_mut(alpha_values, buffer_size as usize)
    }).err()
      .map(|e| e.error)
      .unwrap_or(0)
  } else {
    unsafe {
      tramp(
        this,
        texture_type,
        texture_bounds,
        alpha_values,
        buffer_size,
      )
    }
  }
}

fn copy_texture(
  data: &GlyphRunData,
  texture_type: dw::DWRITE_TEXTURE_TYPE,
  texture_bounds: &RECT,
  alpha_values: &mut [u8],
) -> errors::HResult<()> {
  let texture_width = texture_bounds.right - texture_bounds.left;
  let texture_height = texture_bounds.bottom - texture_bounds.top;
  let GlyphRunData { bounds, bgra_bm } = data;

  let bgra_buf = {
    let mut bgra_buf = vec![0; (4 * texture_width * texture_height) as usize];
    let wic_rc = wincodec::WICRect {
      X: texture_bounds.left - bounds.left,
      Y: texture_bounds.top - bounds.top,
      Width: texture_width,
      Height: texture_height,
    };
    com_invoke!(
      bgra_bm.CopyPixels,
      (&wic_rc),
      ((4 * (bounds.right - bounds.left)) as u32),
      (bgra_buf.len() as u32),
      (bgra_buf.as_mut_ptr())
    )?;
    bgra_buf
  };
  let it = bgra_buf.chunks(4);

  if texture_type == dw::DWRITE_TEXTURE_CLEARTYPE_3x1 {
    alpha_values[0..(texture_width * texture_height * 3) as usize]
      .chunks_mut(3)
      .zip(it)
      .for_each(|(l, r)| {
        l[0] = r[2];
        l[1] = r[1];
        l[2] = r[0];
      });
  } else {
    alpha_values[0..(texture_width * texture_height) as usize]
      .iter_mut()
      .zip(it)
      .for_each(|(l, r)| {
        *l = ((r[0] as u32 + r[1] as u32 + r[2] as u32) / 3) as u8;
      });
  }

  Ok(())
}

pub fn detour_glyph_run_analysis_release(tramp: fns::Release, this: *mut IUnknown) -> ULONG {
  let n = unsafe { tramp(this) };
  if n <= 0 {
    GRA_CTXS
      .write()
      .unwrap()
      .remove(unsafe { &UnsafeSendSync::new(this as *mut dw::IDWriteGlyphRunAnalysis) });
  }
  n
}
