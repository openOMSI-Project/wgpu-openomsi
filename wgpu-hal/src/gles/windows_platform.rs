//! The GLES backend on Windows with the `angle` feature.
//!
//! The backend can create its context with WGL (the system OpenGL driver) or with EGL on
//! [ANGLE](https://github.com/google/angle) (OpenGL ES on Direct3D 11), chosen at instance
//! creation by [`wgt::GlBackendOptions::platform`]. The types here dispatch to the [`wgl`] or
//! the [`egl`] implementation.

use alloc::vec::Vec;
use core::{ffi::c_void, time::Duration};

use windows::Win32::{Foundation, Graphics::Gdi};

use super::{egl, wgl};

/// The backend instance: WGL or ANGLE.
pub enum Instance {
    Wgl(wgl::Instance),
    Angle(egl::Instance),
}

impl Instance {
    /// Returns `true` if this instance runs on ANGLE (EGL on Direct3D 11) instead of WGL.
    pub fn is_angle(&self) -> bool {
        matches!(self, Self::Angle(_))
    }
}

impl crate::Instance for Instance {
    type A = super::Api;

    unsafe fn init(desc: &crate::InstanceDescriptor<'_>) -> Result<Self, crate::InstanceError> {
        match desc.backend_options.gl.platform {
            wgt::GlPlatform::Auto | wgt::GlPlatform::Wgl => {
                unsafe { <wgl::Instance as crate::Instance>::init(desc) }.map(Self::Wgl)
            }
            wgt::GlPlatform::Angle => {
                unsafe { <egl::Instance as crate::Instance>::init(desc) }.map(Self::Angle)
            }
        }
    }

    unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Surface, crate::InstanceError> {
        match self {
            Self::Wgl(instance) => unsafe {
                instance.create_surface(display_handle, window_handle)
            },
            Self::Angle(instance) => unsafe {
                instance.create_surface(display_handle, window_handle)
            },
        }
    }

    unsafe fn enumerate_adapters(
        &self,
        surface_hint: Option<&Surface>,
    ) -> Vec<crate::ExposedAdapter<super::Api>> {
        match self {
            Self::Wgl(instance) => unsafe { instance.enumerate_adapters(surface_hint) },
            Self::Angle(instance) => unsafe { instance.enumerate_adapters(surface_hint) },
        }
    }
}

/// The adapter's GL context, created with WGL or with EGL on ANGLE.
pub enum AdapterContext {
    Wgl(wgl::AdapterContext),
    Angle(egl::AdapterContext),
}

impl AdapterContext {
    /// Returns `true` if this context runs on ANGLE (EGL on Direct3D 11) instead of WGL.
    pub fn is_angle(&self) -> bool {
        matches!(self, Self::Angle(_))
    }

    pub fn is_owned(&self) -> bool {
        match self {
            Self::Wgl(context) => context.is_owned(),
            Self::Angle(context) => context.is_owned(),
        }
    }

    pub fn raw_context(&self) -> *mut c_void {
        match self {
            Self::Wgl(context) => context.raw_context(),
            Self::Angle(context) => context.raw_context(),
        }
    }

    pub(super) fn set_lock_timeout(&mut self, timeout: Duration) {
        match self {
            Self::Wgl(context) => context.set_lock_timeout(timeout),
            Self::Angle(context) => context.set_lock_timeout(timeout),
        }
    }

    /// Obtain a lock to the GL context and get handle to the [`glow::Context`] that can be used to
    /// do rendering.
    ///
    /// Panics if the lock cannot be obtained within the context lock timeout (see
    /// [`wgt::GlBackendOptions::context_lock_timeout`]); [`try_lock`](Self::try_lock) returns an
    /// error instead.
    #[track_caller]
    pub fn lock(&self) -> AdapterContextLock<'_> {
        match self {
            Self::Wgl(context) => AdapterContextLock::Wgl(context.lock()),
            Self::Angle(context) => AdapterContextLock::Angle(context.lock()),
        }
    }

    /// Like [`lock`](Self::lock), but returns [`crate::DeviceError::Lost`] instead of panicking
    /// if the lock cannot be obtained within the context lock timeout.
    #[track_caller]
    pub fn try_lock(&self) -> Result<AdapterContextLock<'_>, crate::DeviceError> {
        match self {
            Self::Wgl(context) => context.try_lock().map(AdapterContextLock::Wgl),
            Self::Angle(context) => context.try_lock().map(AdapterContextLock::Angle),
        }
    }

    /// Locks a WGL context, making it current on `device`. Fails for an ANGLE context, which a
    /// WGL surface cannot use.
    #[track_caller]
    pub(super) fn lock_with_dc(
        &self,
        device: Gdi::HDC,
    ) -> windows::core::Result<AdapterContextLock<'_>> {
        match self {
            Self::Wgl(context) => context.lock_with_dc(device).map(AdapterContextLock::Wgl),
            Self::Angle(_) => {
                log::error!("a WGL surface cannot be used with an ANGLE device");
                Err(windows::core::Error::from_hresult(Foundation::E_INVALIDARG))
            }
        }
    }
}

/// A guard containing a lock to an [`AdapterContext`], while the GL context is kept current.
pub enum AdapterContextLock<'a> {
    Wgl(wgl::AdapterContextLock<'a>),
    Angle(egl::AdapterContextLock<'a>),
}

impl core::ops::Deref for AdapterContextLock<'_> {
    type Target = glow::Context;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Wgl(lock) => lock,
            Self::Angle(lock) => lock,
        }
    }
}

impl super::Adapter {
    pub fn adapter_context(&self) -> &AdapterContext {
        &self.shared.context
    }
}

impl super::Device {
    /// Returns the underlying WGL or EGL (ANGLE) context.
    pub fn context(&self) -> &AdapterContext {
        &self.shared.context
    }
}

enum SurfaceInner {
    Wgl(wgl::Surface),
    Angle(egl::Surface),
}

/// A window surface, presented with WGL or with EGL on ANGLE.
pub struct Surface {
    pub(super) presentable: bool,
    inner: SurfaceInner,
}

impl Surface {
    pub(super) fn wgl(surface: wgl::Surface) -> Self {
        Self {
            presentable: surface.presentable,
            inner: SurfaceInner::Wgl(surface),
        }
    }

    pub(super) fn angle(surface: egl::Surface) -> Self {
        Self {
            presentable: surface.presentable,
            inner: SurfaceInner::Angle(surface),
        }
    }

    pub(super) unsafe fn present(
        &self,
        texture: super::Texture,
        context: &AdapterContext,
    ) -> Result<(), crate::SurfaceError> {
        match (&self.inner, context) {
            (SurfaceInner::Wgl(surface), AdapterContext::Wgl(context)) => unsafe {
                surface.present(texture, context)
            },
            (SurfaceInner::Angle(surface), AdapterContext::Angle(context)) => unsafe {
                surface.present(texture, context)
            },
            _ => Err(crate::SurfaceError::Other(
                "surface and device use different GL platforms (WGL and ANGLE)",
            )),
        }
    }

    pub fn supports_srgb(&self) -> bool {
        match self.inner {
            SurfaceInner::Wgl(ref surface) => surface.supports_srgb(),
            SurfaceInner::Angle(ref surface) => surface.supports_srgb(),
        }
    }
}

impl crate::Surface for Surface {
    type A = super::Api;

    unsafe fn configure(
        &self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), crate::SurfaceError> {
        match (&self.inner, &device.shared.context) {
            (SurfaceInner::Wgl(surface), AdapterContext::Wgl(_)) => unsafe {
                surface.configure(device, config)
            },
            (SurfaceInner::Angle(surface), AdapterContext::Angle(_)) => unsafe {
                surface.configure(device, config)
            },
            _ => Err(crate::SurfaceError::Other(
                "surface and device use different GL platforms (WGL and ANGLE)",
            )),
        }
    }

    unsafe fn unconfigure(&self, device: &super::Device) {
        match self.inner {
            SurfaceInner::Wgl(ref surface) => unsafe { surface.unconfigure(device) },
            SurfaceInner::Angle(ref surface) => unsafe { surface.unconfigure(device) },
        }
    }

    unsafe fn acquire_texture(
        &self,
        timeout: Option<Duration>,
        fence: &super::Fence,
    ) -> Result<crate::AcquiredSurfaceTexture<super::Api>, crate::SurfaceError> {
        match self.inner {
            SurfaceInner::Wgl(ref surface) => unsafe { surface.acquire_texture(timeout, fence) },
            SurfaceInner::Angle(ref surface) => unsafe { surface.acquire_texture(timeout, fence) },
        }
    }

    unsafe fn discard_texture(&self, texture: super::Texture) {
        match self.inner {
            SurfaceInner::Wgl(ref surface) => unsafe { surface.discard_texture(texture) },
            SurfaceInner::Angle(ref surface) => unsafe { surface.discard_texture(texture) },
        }
    }
}
