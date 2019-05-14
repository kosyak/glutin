use crate::api::egl::{
    Context as EglContext, NativeDisplay, SurfaceType as EglSurfaceType,
};
use crate::{
    ContextError, CreationError, GlAttributes, PixelFormat,
    PixelFormatRequirements,
};

use crate::os::unix::{EventsLoopExt, WindowExt};
use glutin_egl_sys as ffi;
use winit;
use winit::dpi;
use winit::{Window, WindowBuilder};

use std::ops::Deref;
use std::os::raw;

#[derive(Debug)]
pub enum Context {
    Windowed(EglContext),
    PBuffer(EglContext),
    Surfaceless(EglContext),
}

impl Deref for Context {
    type Target = EglContext;

    fn deref(&self) -> &Self::Target {
        match self {
            Context::Windowed(ctx) => ctx,
            Context::PBuffer(ctx) => ctx,
            Context::Surfaceless(ctx) => ctx,
        }
    }
}

impl Context {
    #[inline]
    pub fn new_headless(
        el: &winit::EventsLoop,
        pf_reqs: &PixelFormatRequirements,
        gl_attr: &GlAttributes<&Context>,
        size: Option<dpi::PhysicalSize>,
    ) -> Result<Self, CreationError> {
        let gl_attr = gl_attr.clone().map_sharing(|c| &**c);
        let display_ptr = el.get_gbm_display().unwrap() as *const _;
        let native_display =
            NativeDisplay::Gbm(Some(display_ptr as *const _));
        if let Some(size) = size {
            let context = EglContext::new(
                pf_reqs,
                &gl_attr,
                native_display,
                EglSurfaceType::PBuffer
            )
            .and_then(|p| p.finish_pbuffer(size))?;
            let context = Context::PBuffer(context);
            Ok(context)
        } else {
            // Surfaceless
            let context = EglContext::new(
                pf_reqs,
                &gl_attr,
                native_display,
                EglSurfaceType::Surfaceless
            )
            .and_then(|p| p.finish_surfaceless())?;
            let context = Context::Surfaceless(context);
            Ok(context)
        }
    }

    #[inline]
    pub fn new(
        wb: WindowBuilder,
        el: &winit::EventsLoop,
        pf_reqs: &PixelFormatRequirements,
        gl_attr: &GlAttributes<&Context>,
    ) -> Result<(Window, Self), CreationError> {
        let win = wb.build(el)?;

        let dpi_factor = win.get_hidpi_factor();
        let size = win.get_inner_size().unwrap().to_physical(dpi_factor);
        let (width, height): (u32, u32) = size.into();

        let display_ptr = win.get_gbm_display().unwrap() as *const _;
        let surface = win.get_gbm_surface();
        let surface = match surface {
            Some(s) => s,
            None => {
                return Err(CreationError::NotSupported(
                    "Gbm not found".to_string(),
                ));
            }
        };

        let context = Self::new_raw_context(
            display_ptr,
            surface,
            width,
            height,
            pf_reqs,
            gl_attr,
        )?;
        Ok((win, context))
    }

    #[inline]
    pub fn new_raw_context(
        display_ptr: *const raw::c_void,
        surface: *const raw::c_void,
        _width: u32,
        _height: u32,
        pf_reqs: &PixelFormatRequirements,
        gl_attr: &GlAttributes<&Context>,
    ) -> Result<Self, CreationError> {
        let context = {
            let gl_attr = gl_attr.clone().map_sharing(|c| &**c);
            let native_display =
                NativeDisplay::Gbm(Some(display_ptr as *const _));
            EglContext::new(
                pf_reqs,
                &gl_attr,
                native_display,
                EglSurfaceType::Window
            )
            .and_then(|p| p.finish(surface))?
        };
        let context = Context::Windowed(context);
        Ok(context)
    }

    #[inline]
    pub unsafe fn make_current(&self) -> Result<(), ContextError> {
        (**self).make_current()
    }

    #[inline]
    pub unsafe fn make_not_current(&self) -> Result<(), ContextError> {
        (**self).make_not_current()
    }

    #[inline]
    pub fn is_current(&self) -> bool {
        (**self).is_current()
    }

    #[inline]
    pub fn get_api(&self) -> crate::Api {
        (**self).get_api()
    }

    #[inline]
    pub unsafe fn raw_handle(&self) -> ffi::EGLContext {
        (**self).raw_handle()
    }

    #[inline]
    pub unsafe fn get_egl_display(&self) -> Option<*const raw::c_void> {
        Some((**self).get_egl_display())
    }

    #[inline]
    pub fn resize(&self, _width: u32, _height: u32) {
        match self {
            // Context::Windowed(_, surface) => {
            //     surface.0.resize(width as i32, height as i32, 0, 0)
            // }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const () {
        (**self).get_proc_address(addr)
    }

    #[inline]
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        (**self).swap_buffers()
    }

    #[inline]
    pub fn get_pixel_format(&self) -> PixelFormat {
        (**self).get_pixel_format().clone()
    }
}

