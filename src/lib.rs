#![cfg(windows)]
#![recursion_limit = "256"]
#[macro_use]
extern crate detour;
extern crate encoding;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate winapi;
extern crate wio;

use detour::StaticDetour;
use std::{
  ffi::{CString, OsString},
  mem, ptr,
  sync::Mutex,
};
use winapi::{
  shared::{
    basetsd::UINT32,
    minwindef::{BOOL, BYTE, DWORD, FALSE, FLOAT, HINSTANCE, LPVOID, TRUE, ULONG},
    windef,
    winerror::HRESULT,
  },
  um::{
    combaseapi, consoleapi, d2d1, dcommon, dwrite as dw, dwrite_1 as dw_1, dwrite_2 as dw_2,
    dwrite_3 as dw_3, libloaderapi, unknwnbase::IUnknown, wincodec,
  },
  Interface,
};
use wio::{com::ComPtr, wide::ToWide};

#[macro_use]
pub mod errors;
pub mod config;
pub mod d2d1_helper;
pub mod dwrite;
pub mod fns;
pub mod gdi;
pub mod util;

#[allow(dead_code)]
struct Detours {
  d_create_glyph_run_analysis: StaticDetour<fns::CreateGlyphRunAnalysis>,
  d_create_glyph_run_analysis_2: StaticDetour<fns::CreateGlyphRunAnalysis2>,
  d_create_glyph_run_analysis_3: StaticDetour<fns::CreateGlyphRunAnalysis3>,
  d_create_alpha_texture: StaticDetour<fns::CreateAlphaTexture>,
  d_get_alpha_texture_bounds: StaticDetour<fns::GetAlphaTextureBounds>,
  d_glyph_run_analysis_release: StaticDetour<fns::Release>,
  d_ext_text_out_w: StaticDetour<fns::ExtTextOutW>,
  d_text_out_w: StaticDetour<fns::TextOutW>,
}

lazy_static! {
  static ref DETOURS: Mutex<Option<Detours>> = Mutex::new(None);
}

static_detours! {
  struct DetourCreateGlyphRunAnalysis: unsafe extern "system" fn (
    *mut dw::IDWriteFactory,
    *const dw::DWRITE_GLYPH_RUN,
    FLOAT,
    *const dw::DWRITE_MATRIX,
    dw::DWRITE_RENDERING_MODE,
    dcommon::DWRITE_MEASURING_MODE,
    FLOAT,
    FLOAT,
    *mut *mut dw::IDWriteGlyphRunAnalysis
  ) -> HRESULT;

  struct DetourCreateGlyphRunAnalysis2: unsafe extern "system" fn (
    *mut dw_2::IDWriteFactory2,
    *const dw::DWRITE_GLYPH_RUN,
    *const dw::DWRITE_MATRIX,
    dw::DWRITE_RENDERING_MODE,
    dw_2::DWRITE_GRID_FIT_MODE,
    dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
    dcommon::DWRITE_MEASURING_MODE,
    FLOAT,
    FLOAT,
    *mut *mut dw::IDWriteGlyphRunAnalysis
  ) -> HRESULT;

  struct DetourCreateGlyphRunAnalysis3: unsafe extern "system" fn (
    *mut dw_3::IDWriteFactory3,
    *const dw::DWRITE_GLYPH_RUN,
    *const dw::DWRITE_MATRIX,
    dw_3::DWRITE_RENDERING_MODE1,
    dw_2::DWRITE_GRID_FIT_MODE,
    dw_1::DWRITE_TEXT_ANTIALIAS_MODE,
    dcommon::DWRITE_MEASURING_MODE,
    FLOAT,
    FLOAT,
    *mut *mut dw::IDWriteGlyphRunAnalysis
  ) -> HRESULT;

  struct DetourCreateAlphaTexture: unsafe extern "system" fn (
    *mut dw::IDWriteGlyphRunAnalysis,
    dw::DWRITE_TEXTURE_TYPE,
    *const windef::RECT,
    *mut BYTE,
    UINT32
  ) -> HRESULT;

  struct DetourGetAlphaTextureBounds: unsafe extern "system" fn (
    *mut dw::IDWriteGlyphRunAnalysis,
    dw::DWRITE_TEXTURE_TYPE,
    *mut windef::RECT
  ) -> HRESULT;

  struct DetourGlyphRunAnalysisRelease: unsafe extern "system" fn (
    *mut IUnknown
  ) -> ULONG;

  struct DetourExtTextOutW: unsafe extern "system" fn (
    windef::HDC,
    i32,
    i32,
    u32,
    *const windef::RECT,
    *const u16,
    u32,
    *const i32
  ) -> i32;

  struct DetourTextOutW: unsafe extern "system" fn (
    windef::HDC,
    i32,
    i32,
    *const u16,
    i32
  ) -> i32;
}

fn run() -> errors::HResult<()> {
  com_invoke!(
    (combaseapi::CoInitializeEx),
    (ptr::null_mut()),
    (combaseapi::COINITBASE_MULTITHREADED)
  )?;
  let dw_fac = unsafe {
    ComPtr::from_raw(
      com_invoke!(
    (dw::DWriteCreateFactory),
    (dw::DWRITE_FACTORY_TYPE_SHARED),
    (&dw::IDWriteFactory::uuidof()),
    (-> p)
  )?.0 as *mut dw::IDWriteFactory,
    )
  };
  let dw_fac_2 = unsafe {
    ComPtr::from_raw(
      com_invoke!(
    (dw::DWriteCreateFactory),
    (dw::DWRITE_FACTORY_TYPE_SHARED),
    (&dw_2::IDWriteFactory2::uuidof()),
    (-> p)
  )?.0 as *mut dw_2::IDWriteFactory2,
    )
  };
  let dw_fac_3 = unsafe {
    ComPtr::from_raw(
      com_invoke!(
    (dw::DWriteCreateFactory),
    (dw::DWRITE_FACTORY_TYPE_SHARED),
    (&dw_3::IDWriteFactory3::uuidof()),
    (-> p)
  )?.0 as *mut dw_3::IDWriteFactory3,
    )
  };
  let wic_fac = unsafe {
    ComPtr::from_raw(
      com_invoke!(
        (combaseapi::CoCreateInstance),
        (&wincodec::CLSID_WICImagingFactory),
        (ptr::null_mut()),
        (combaseapi::CLSCTX_INPROC),
        (&wincodec::IWICImagingFactory::uuidof()),
        (-> p)
      )?.0 as *mut wincodec::IWICImagingFactory,
    )
  };
  let d2d_fac = unsafe {
    ComPtr::from_raw(
      com_invoke!(
        (d2d1::D2D1CreateFactory),
        (d2d1::D2D1_FACTORY_TYPE_MULTI_THREADED),
        (&d2d1::ID2D1Factory::uuidof()),
        (ptr::null()),
        (-> p)
      )?.0 as *mut d2d1::ID2D1Factory,
    )
  };
  let (dw_gdi, ()) = com_invoke!(
    dw_fac.GetGdiInterop,
    (->> p)
  )?;

  let (collection, ()) = com_invoke!(dw_fac.GetSystemFontCollection, (->> p), FALSE).unwrap();
  let (fam_idx, (fam_exists, ())) = com_invoke!(
    collection.FindFamilyName,
    (OsString::from("Segoe UI".to_string()).to_wide_null().as_ptr()),
    (-> i),
    (-> e)
  )?;
  if fam_exists != TRUE {
    return Err(annotate_error!(0));
  }
  let (fam, ()) = com_invoke!(collection.GetFontFamily, fam_idx, (->> p))?;
  let (font, ()) = com_invoke!(
    fam.GetFirstMatchingFont,
    (dw::DWRITE_FONT_WEIGHT_NORMAL),
    (dw::DWRITE_FONT_STRETCH_NORMAL),
    (dw::DWRITE_FONT_STYLE_NORMAL),
    (->> p)
  )?;
  let (face, ()) = com_invoke!(font.CreateFontFace, (->> p))?;

  let gr = unsafe {
    dw::DWRITE_GLYPH_RUN {
      fontFace: face.as_raw(),
      fontEmSize: 16.0,
      glyphIndices: (&[42u16] as &'static [u16]).as_ptr(),
      glyphCount: 1,
      ..mem::zeroed()
    }
  };
  let (gla, ()) = com_invoke!(
    dw_fac.CreateGlyphRunAnalysis,
    (&gr),
    1.0,
    (ptr::null()),
    (dw::DWRITE_RENDERING_MODE_ALIASED),
    (dcommon::DWRITE_MEASURING_MODE_NATURAL),
    0.0,
    0.0,
    (->> p)
  )?;

  unsafe {
    let mut d_get_alpha_texture_bounds = DetourGetAlphaTextureBounds
      .initialize(
        (*(*gla.as_raw()).lpVtbl).GetAlphaTextureBounds,
        dwrite::detour_get_alpha_texture_bounds,
      ).unwrap();
    d_get_alpha_texture_bounds.enable().unwrap();

    let mut d_create_glyph_run_analysis_2 = DetourCreateGlyphRunAnalysis2
      .initialize((*(*dw_fac_2.as_raw()).lpVtbl).CreateGlyphRunAnalysis, {
        let wic_fac = util::UnsafeSendSync::new(wic_fac.clone());
        let d2d_fac = util::UnsafeSendSync::new(d2d_fac.clone());
        let get_alpha_texture_bounds = mem::transmute(d_get_alpha_texture_bounds.trampoline());
        move |tramp,
              this,
              glyph_run,
              transform,
              rendering_mode,
              measuring_mode,
              grid_fit_mode,
              antialias_mode,
              baseline_origin_x,
              baseline_origin_y,
              glyph_run_analysis| {
          dwrite::detour_create_glyph_run_analysis_2(
            wic_fac.as_ref().as_raw(),
            d2d_fac.as_ref().as_raw(),
            get_alpha_texture_bounds,
            tramp,
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
        }
      }).unwrap();
    d_create_glyph_run_analysis_2.enable().unwrap();

    let mut d_create_glyph_run_analysis_3 = DetourCreateGlyphRunAnalysis3
      .initialize((*(*dw_fac_3.as_raw()).lpVtbl).CreateGlyphRunAnalysis, {
        let wic_fac = util::UnsafeSendSync::new(wic_fac.clone());
        let d2d_fac = util::UnsafeSendSync::new(d2d_fac.clone());
        let get_alpha_texture_bounds = mem::transmute(d_get_alpha_texture_bounds.trampoline());
        move |tramp,
              this,
              glyph_run,
              transform,
              rendering_mode,
              measuring_mode,
              grid_fit_mode,
              antialias_mode,
              baseline_origin_x,
              baseline_origin_y,
              glyph_run_analysis| {
          dwrite::detour_create_glyph_run_analysis_3(
            wic_fac.as_ref().as_raw(),
            d2d_fac.as_ref().as_raw(),
            get_alpha_texture_bounds,
            tramp,
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
        }
      }).unwrap();
    d_create_glyph_run_analysis_3.enable().unwrap();

    let mut d_create_glyph_run_analysis = DetourCreateGlyphRunAnalysis
      .initialize((*(*dw_fac.as_raw()).lpVtbl).CreateGlyphRunAnalysis, {
        let dw_fac_3 = util::UnsafeSendSync::new(dw_fac_3.clone());
        let wic_fac = util::UnsafeSendSync::new(wic_fac.clone());
        let d2d_fac = util::UnsafeSendSync::new(d2d_fac.clone());
        let get_alpha_texture_bounds = mem::transmute(d_get_alpha_texture_bounds.trampoline());
        let create_glyph_run_analysis_3 =
          mem::transmute(d_create_glyph_run_analysis_3.trampoline());
        move |_,
              this,
              glyph_run,
              ppd,
              transform,
              rendering_mode,
              measuring_mode,
              baseline_x,
              baseline_y,
              analysis| {
          dwrite::detour_create_glyph_run_analysis(
            dw_fac_3.as_ref().as_raw(),
            wic_fac.as_ref().as_raw(),
            d2d_fac.as_ref().as_raw(),
            get_alpha_texture_bounds,
            create_glyph_run_analysis_3,
            this,
            glyph_run,
            ppd,
            transform,
            rendering_mode,
            measuring_mode,
            baseline_x,
            baseline_y,
            analysis,
          )
        }
      }).unwrap();
    d_create_glyph_run_analysis.enable().unwrap();

    let mut d_create_alpha_texture = DetourCreateAlphaTexture
      .initialize((*(*gla.as_raw()).lpVtbl).CreateAlphaTexture, {
        move |tramp, this, texture_type, texture_bounds, alpha_values, buffer_size| {
          dwrite::detour_create_alpha_texture(
            tramp,
            this,
            texture_type,
            texture_bounds,
            alpha_values,
            buffer_size,
          )
        }
      }).unwrap();
    d_create_alpha_texture.enable().unwrap();

    let mut d_glyph_run_analysis_release = DetourGlyphRunAnalysisRelease
      .initialize(
        (*(*gla.as_raw()).lpVtbl).parent.Release,
        dwrite::detour_glyph_run_analysis_release,
      ).unwrap();
    d_glyph_run_analysis_release.enable().unwrap();

    let h_gdi32 = {
      let gdi32 = CString::new("gdi32.dll").unwrap();
      libloaderapi::GetModuleHandleA(gdi32.as_ptr())
    };
    let TextOutW = {
      let s = CString::new("TextOutW").unwrap();
      libloaderapi::GetProcAddress(h_gdi32, s.as_ptr())
    };
    let ExtTextOutW = {
      let s = CString::new("ExtTextOutW").unwrap();
      libloaderapi::GetProcAddress(h_gdi32, s.as_ptr())
    };

    let mut d_ext_text_out_w = DetourExtTextOutW
      .initialize(mem::transmute(ExtTextOutW), {
        let dw_fac_3 = util::UnsafeSendSync::new(dw_fac_3.clone());
        let d2d_fac = util::UnsafeSendSync::new(d2d_fac.clone());
        let dw_gdi = util::UnsafeSendSync::new(dw_gdi.clone());
        let create_glyph_run_analysis = mem::transmute(d_create_glyph_run_analysis_3.trampoline());
        let get_alpha_texture_bounds = mem::transmute(d_get_alpha_texture_bounds.trampoline());

        move |tramp, hdc, x, y, options, rect, s, c, dxs| {
          gdi::ext_text_out_w(
            dw_fac_3.as_ref().as_raw(),
            d2d_fac.as_ref().as_raw(),
            dw_gdi.as_ref().as_raw(),
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
          )
        }
      }).unwrap();
    d_ext_text_out_w.enable().unwrap();

    let mut d_text_out_w = DetourTextOutW
      .initialize(mem::transmute(TextOutW), gdi::text_out_w)
      .unwrap();
    d_text_out_w.enable().unwrap();

    *DETOURS.lock().unwrap() = Some(Detours {
      d_create_alpha_texture,
      d_create_glyph_run_analysis,
      d_create_glyph_run_analysis_2,
      d_create_glyph_run_analysis_3,
      d_get_alpha_texture_bounds,
      d_glyph_run_analysis_release,
      d_ext_text_out_w,
      d_text_out_w,
    })
  }

  Ok(())
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(_: HINSTANCE, reason: DWORD, _: LPVOID) -> BOOL {
  match reason {
    0 => {
      let _ = DETOURS.lock().map(|mut v| v.take());
      TRUE
    }
    1 => match run() {
      Ok(_) => {
        unsafe { consoleapi::AllocConsole() };
        println!("ok?");
        TRUE
      }
      _ => FALSE,
    },
    _ => TRUE,
  }
}
