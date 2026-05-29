//! WGPU Initialization Service

use super::{BACKEND, InitHints, Shader, WgpuLink, WrappedContext};
use crate::{
    BResult, gamestate::BTerm, hal::Framebuffer, hal::scaler::ScreenScaler,
    prelude::BACKEND_INTERNAL,
};
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::{event_loop::EventLoop, window::Window};

pub fn init_raw<S: ToString>(
    width_pixels: u32,
    height_pixels: u32,
    window_title: S,
    platform_hints: InitHints,
) -> BResult<BTerm> {
    let mut scaler = ScreenScaler::new(platform_hints.desired_gutter, width_pixels, height_pixels);
    let el = EventLoop::new()?;
    let wb = Window::default_attributes()
        .with_title(window_title.to_string())
        .with_min_inner_size(scaler.new_window_size())
        .with_inner_size(scaler.new_window_size());

    #[allow(deprecated)]
    let window = Arc::new(el.create_window(wb)?);

    let (instance, surface, adapter, device, queue, config) =
        pollster::block_on(init_adapter(window.clone()))?;

    // Shaders
    let mut shaders: Vec<Shader> = Vec::new();
    shaders.push(Shader::new(
        &device,
        include_str!("shader_source/console_with_bg.wgsl"),
    ));
    shaders.push(Shader::new(
        &device,
        include_str!("shader_source/console_no_bg.wgsl"),
    ));
    shaders.push(Shader::new(
        &device,
        include_str!("shader_source/backing_plain.wgsl"),
    ));
    shaders.push(Shader::new(
        &device,
        include_str!("shader_source/fancy.wgsl"),
    ));
    shaders.push(Shader::new(
        &device,
        include_str!("shader_source/sprites.wgsl"),
    ));

    BACKEND_INTERNAL.lock().shaders = shaders;

    // Build the backing frame-buffer
    let initial_dpi_factor = window.scale_factor();
    scaler.change_logical_size(width_pixels, height_pixels, initial_dpi_factor as f32);
    let backing_buffer = Framebuffer::new(
        &device,
        config.format,
        scaler.logical_size.0,
        scaler.logical_size.1,
    );

    // Build a simple quad rendering VAO
    //let quad_vao = setup_quad(&gl);

    // Store the backend
    let mut be = BACKEND.lock();
    be.context_wrapper = Some(WrappedContext { el, window });
    be.wgpu = Some(WgpuLink {
        instance,
        surface,
        adapter,
        device,
        queue,
        config,
        backing_buffer,
    });
    be.frame_sleep_time = crate::hal::convert_fps_to_wait(platform_hints.frame_sleep_time);
    be.resize_scaling = platform_hints.resize_scaling;
    be.screen_scaler = scaler;

    let bterm = BTerm {
        width_pixels,
        height_pixels,
        original_width_pixels: width_pixels,
        original_height_pixels: height_pixels,
        fps: 0.0,
        frame_time_ms: 0.0,
        active_console: 0,
        key: None,
        mouse_pos: (0, 0),
        left_click: false,
        shift: false,
        control: false,
        alt: false,
        web_button: None,
        quitting: false,
        post_scanlines: false,
        post_screenburn: false,
        screen_burn_color: bracket_color::prelude::RGB::from_f32(0.0, 1.0, 1.0),
        mouse_visible: true,
    };
    Ok(bterm)
}

async fn init_adapter(
    window: Arc<Window>,
) -> BResult<(
    Instance,
    Surface<'static>,
    Adapter,
    Device,
    Queue,
    SurfaceConfiguration,
)> {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let surface = instance.create_surface(window)?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        })
        .await?;

    //println!("{:?}", adapter.get_info());
    let config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or_else(|| "Failed to create default webgpu surface configuration".to_string())?;
    surface.configure(&device, &config);

    Ok((instance, surface, adapter, device, queue, config))
}
