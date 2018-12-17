use winapi::{
  shared::{
    minwindef::{FLOAT, ULONG},
    windef,
  },
  um::{
    dcommon, dwrite as dw, dwrite_1 as dw_1, dwrite_2 as dw_2, dwrite_3 as dw_3,
    unknwnbase::IUnknown, winnt::HRESULT,
  },
};

pub type CreateGlyphRunAnalysis3 = unsafe extern "system" fn(
  *mut dw_3::IDWriteFactory3,
  *const dw::DWRITE_GLYPH_RUN,
  *const dw::DWRITE_MATRIX,
  dw_3::DWRITE_RENDERING_MODE1,
  dcommon::DWRITE_MEASURING_MODE,
  dw_2::DWRITE_GRID_FIT_MODE,
  dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
  FLOAT,
  FLOAT,
  *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT;

pub type CreateGlyphRunAnalysis2 = unsafe extern "system" fn(
  *mut dw_2::IDWriteFactory2,
  *const dw::DWRITE_GLYPH_RUN,
  *const dw::DWRITE_MATRIX,
  dw::DWRITE_RENDERING_MODE,
  dcommon::DWRITE_MEASURING_MODE,
  dw_2::DWRITE_GRID_FIT_MODE,
  dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
  FLOAT,
  FLOAT,
  *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT;

pub type CreateGlyphRunAnalysis = unsafe extern "system" fn(
  *mut dw::IDWriteFactory,
  *const dw::DWRITE_GLYPH_RUN,
  FLOAT,
  *const dw::DWRITE_MATRIX,
  dw::DWRITE_RENDERING_MODE,
  dcommon::DWRITE_MEASURING_MODE,
  FLOAT,
  FLOAT,
  *mut *mut dw::IDWriteGlyphRunAnalysis,
) -> HRESULT;

pub type GetAlphaTextureBounds = unsafe extern "system" fn(
  *mut dw::IDWriteGlyphRunAnalysis,
  dw::DWRITE_TEXTURE_TYPE,
  *mut windef::RECT,
) -> HRESULT;

pub type Release = unsafe extern "system" fn(*mut IUnknown) -> ULONG;

pub type CreateAlphaTexture = unsafe extern "system" fn(
  *mut dw::IDWriteGlyphRunAnalysis,
  dw::DWRITE_TEXTURE_TYPE,
  *const windef::RECT,
  *mut u8,
  u32,
) -> HRESULT;

pub type ExtTextOutW = unsafe extern "system" fn(
  windef::HDC,
  i32,
  i32,
  u32,
  *const windef::RECT,
  *const u16,
  u32,
  *const i32,
) -> i32;

pub type TextOutW = unsafe extern "system" fn(windef::HDC, i32, i32, *const u16, i32) -> i32;
