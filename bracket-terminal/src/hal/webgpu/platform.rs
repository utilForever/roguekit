//! WGPU Platform definition

use super::Framebuffer;
use crate::hal::scaler::{ScreenScaler, default_gutter_size};
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::{event_loop::EventLoop, window::Window};

/// Defines the WGPU platform
pub struct PlatformGL<'a> {
    /// Wrapper for the winit context
    pub context_wrapper: Option<WrappedContext>,
    /// Contains the WGPU back-end (device, etc.)
    pub wgpu: Option<WgpuLink<'a>>,

    /// Target delay per frame
    pub frame_sleep_time: Option<u64>,
    /// Should the back-end resize windows by character (true) or just scale them (false)?
    pub resize_scaling: bool,
    /// Is there a request to resize the console?
    pub resize_request: Option<(u32, u32)>,
    /// Are we requesting a screenshot?
    pub request_screenshot: Option<String>,
    /// Screen scaling system
    pub screen_scaler: ScreenScaler,
}

pub struct WgpuLink<'a> {
    pub instance: Instance,
    pub surface: Surface<'a>,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub backing_buffer: Framebuffer,
}

unsafe impl<'a> Send for PlatformGL<'a> {}
unsafe impl<'a> Sync for PlatformGL<'a> {}

pub struct WrappedContext {
    pub el: EventLoop<()>,
    pub window: Arc<Window>,
}

pub struct InitHints {
    pub vsync: bool,
    pub fullscreen: bool,
    pub frame_sleep_time: Option<f32>,
    pub resize_scaling: bool,
    pub desired_gutter: u32,
    pub fitscreen: bool,
}

impl InitHints {
    pub fn new() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            frame_sleep_time: None,
            resize_scaling: false,
            desired_gutter: default_gutter_size(),
            fitscreen: false,
        }
    }
}
