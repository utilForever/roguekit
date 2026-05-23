use super::BACKEND;
use crate::BResult;
use crate::hal::native::{NativeInitSettings, WrappedContext, shader_strings};
use crate::hal::scaler::ScreenScaler;
use crate::hal::{Framebuffer, Shader, setup_quad};
use crate::prelude::{BACKEND_INTERNAL, BTerm, InitHints};
use glow::HasContext;
use glutin::{
    config::{ConfigSurfaceTypes, ConfigTemplateBuilder},
    context::ContextAttributesBuilder,
    display::{DisplayApiPreference, GetGlDisplay},
    prelude::*,
    surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use std::ffi::CString;
use std::num::NonZeroU32;
use winit::{
    dpi::LogicalSize,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Fullscreen, Window, WindowAttributes},
};

pub struct NativeRuntime {
    pub window: Window,
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub gl_surface: glutin::surface::Surface<WindowSurface>,
}

pub fn init_raw<S: ToString>(
    width_pixels: u32,
    height_pixels: u32,
    window_title: S,
    platform_hints: InitHints,
) -> BResult<BTerm> {
    let el = EventLoop::new()?;
    let frame_sleep_time = crate::hal::convert_fps_to_wait(platform_hints.frame_sleep_time);
    let resize_scaling = platform_hints.resize_scaling;
    let scaler = ScreenScaler::new(platform_hints.desired_gutter, width_pixels, height_pixels);
    {
        let mut be = BACKEND.lock();
        be.context_wrapper = Some(WrappedContext {
            el,
            init: NativeInitSettings {
                width_pixels,
                height_pixels,
                window_title: window_title.to_string(),
                platform_hints,
            },
        });
        be.frame_sleep_time = frame_sleep_time;
        be.resize_scaling = resize_scaling;
        be.screen_scaler = scaler;
    }

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

pub(super) fn init_runtime(
    event_loop: &ActiveEventLoop,
    init: NativeInitSettings,
) -> BResult<NativeRuntime> {
    let NativeInitSettings {
        width_pixels,
        height_pixels,
        window_title,
        platform_hints,
    } = init;
    let InitHints {
        vsync,
        fullscreen,
        gl_version,
        gl_profile,
        hardware_acceleration,
        srgb,
        frame_sleep_time,
        resize_scaling,
        desired_gutter,
        fitscreen,
    } = platform_hints;

    let mut scaler = ScreenScaler::new(desired_gutter, width_pixels, height_pixels);
    let window_size = scaler.new_window_size();
    let window_size = LogicalSize::new(window_size.width, window_size.height);
    let window_attributes = WindowAttributes::default()
        .with_title(window_title)
        .with_resizable(fitscreen)
        .with_min_inner_size(window_size)
        .with_inner_size(window_size);
    let window = event_loop.create_window(window_attributes)?;

    let raw_display = window.display_handle()?.as_raw();
    #[cfg(target_os = "macos")]
    let preference = DisplayApiPreference::Cgl;
    #[cfg(target_os = "windows")]
    let preference = DisplayApiPreference::EglThenWgl(Some(window.window_handle()?.as_raw()));
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let preference = DisplayApiPreference::Egl;

    let gl_display = unsafe { glutin::display::Display::new(raw_display, preference)? };

    let mut template_builder = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(false)
        .with_surface_type(ConfigSurfaceTypes::WINDOW);
    if hardware_acceleration {
        template_builder = template_builder.prefer_hardware_accelerated(Some(true));
    }

    let raw_window_handle = window.window_handle()?.as_raw();
    let primary_template = template_builder
        .clone()
        .compatible_with_native_window(raw_window_handle)
        .build();
    let fallback_template = template_builder.build();
    let config = unsafe {
        gl_display
            .find_configs(primary_template)?
            .next()
            .or_else(|| gl_display.find_configs(fallback_template).ok()?.next())
            .ok_or("No compatible GL configuration found")?
    };

    let context_attributes = ContextAttributesBuilder::new()
        .with_profile(gl_profile)
        .with_context_api(gl_version)
        .build(Some(raw_window_handle));
    let not_current_gl_context =
        unsafe { gl_display.create_context(&config, &context_attributes)? };

    let physical_size = window.inner_size();
    let width = NonZeroU32::new(physical_size.width.max(1)).unwrap();
    let height = NonZeroU32::new(physical_size.height.max(1)).unwrap();
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(srgb))
        .build(raw_window_handle, width, height);
    let gl_surface = unsafe { gl_display.create_window_surface(&config, &attrs)? };
    let gl_context = not_current_gl_context.make_current(&gl_surface)?;

    if vsync {
        let _ = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));
    } else {
        let _ = gl_surface.set_swap_interval(&gl_context, SwapInterval::DontWait);
    }

    if fullscreen {
        if let Some(mh) = event_loop.available_monitors().next() {
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(mh))));
        } else {
            return Err("No available monitor found".into());
        }
    }

    let gl = unsafe {
        glow::Context::from_loader_function(|ptr| {
            let symbol = CString::new(ptr).unwrap();
            gl_context.display().get_proc_address(&symbol) as *const _
        })
    };

    #[cfg(debug_assertions)]
    unsafe {
        let gl_version = gl.get_parameter_string(glow::VERSION);
        let shader_version = gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION);
        println!(
            "Initialized OpenGL with: {}, Shader Language Version: {}",
            gl_version, shader_version
        );
    }

    let shaders: Vec<Shader> = vec![
        Shader::new(
            &gl,
            shader_strings::CONSOLE_WITH_BG_VS,
            shader_strings::CONSOLE_WITH_BG_FS,
        ),
        Shader::new(
            &gl,
            shader_strings::CONSOLE_NO_BG_VS,
            shader_strings::CONSOLE_NO_BG_FS,
        ),
        Shader::new(&gl, shader_strings::BACKING_VS, shader_strings::BACKING_FS),
        Shader::new(
            &gl,
            shader_strings::SCANLINES_VS,
            shader_strings::SCANLINES_FS,
        ),
        Shader::new(
            &gl,
            shader_strings::FANCY_CONSOLE_VS,
            shader_strings::FANCY_CONSOLE_FS,
        ),
        Shader::new(
            &gl,
            shader_strings::SPRITE_CONSOLE_VS,
            shader_strings::SPRITE_CONSOLE_FS,
        ),
    ];

    let initial_dpi_factor = window.scale_factor();
    scaler.change_logical_size(width_pixels, height_pixels, initial_dpi_factor as f32);
    let backing_fbo = Framebuffer::build_fbo(
        &gl,
        scaler.logical_size.0 as i32,
        scaler.logical_size.1 as i32,
    )?;
    let quad_vao = setup_quad(&gl);

    let mut be = BACKEND.lock();
    be.gl = Some(gl);
    be.quad_vao = Some(quad_vao);
    be.backing_buffer = Some(backing_fbo);
    be.frame_sleep_time = crate::hal::convert_fps_to_wait(frame_sleep_time);
    be.resize_scaling = resize_scaling;
    be.screen_scaler = scaler;

    BACKEND_INTERNAL.lock().shaders = shaders;

    Ok(NativeRuntime {
        window,
        gl_context,
        gl_surface,
    })
}
