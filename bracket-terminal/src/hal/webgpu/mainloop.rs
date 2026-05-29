//! WGPU Main Loop

use super::{
    CONSOLE_BACKING, ConsoleBacking, FancyConsoleBackend, Font, Framebuffer, SimpleConsoleBackend,
    SparseConsoleBackend, SpriteConsoleBackend, WgpuLink, quadrender::QuadRender,
};
use crate::{
    BResult,
    gamestate::{BTerm, GameState},
    hal::scaler::FontScaler,
    input::{BEvent, clear_input_state},
    prelude::{
        BACKEND, BACKEND_INTERNAL, FlexiConsole, INPUT, SimpleConsole, SparseConsole, SpriteConsole,
    },
};
use bracket_geometry::prelude::Point;
use std::mem::size_of;
use std::{rc::Rc, time::Instant};
use wgpu::TextureViewDescriptor;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

const TICK_TYPE: ControlFlow = ControlFlow::Poll;

struct ResizeEvent {
    physical_size: PhysicalSize<u32>,
    dpi_scale_factor: f64,
    send_event: bool,
}

// Translate winit 0.30 key codes into the existing engine's VirtualKeyCode values.
fn map_keycode(code: KeyCode) -> Option<crate::hal::VirtualKeyCode> {
    use crate::hal::VirtualKeyCode as V;
    use KeyCode as W;

    Some(match code {
        W::Digit0 => V::Key0,
        W::Digit1 => V::Key1,
        W::Digit2 => V::Key2,
        W::Digit3 => V::Key3,
        W::Digit4 => V::Key4,
        W::Digit5 => V::Key5,
        W::Digit6 => V::Key6,
        W::Digit7 => V::Key7,
        W::Digit8 => V::Key8,
        W::Digit9 => V::Key9,
        W::KeyA => V::A,
        W::KeyB => V::B,
        W::KeyC => V::C,
        W::KeyD => V::D,
        W::KeyE => V::E,
        W::KeyF => V::F,
        W::KeyG => V::G,
        W::KeyH => V::H,
        W::KeyI => V::I,
        W::KeyJ => V::J,
        W::KeyK => V::K,
        W::KeyL => V::L,
        W::KeyM => V::M,
        W::KeyN => V::N,
        W::KeyO => V::O,
        W::KeyP => V::P,
        W::KeyQ => V::Q,
        W::KeyR => V::R,
        W::KeyS => V::S,
        W::KeyT => V::T,
        W::KeyU => V::U,
        W::KeyV => V::V,
        W::KeyW => V::W,
        W::KeyX => V::X,
        W::KeyY => V::Y,
        W::KeyZ => V::Z,
        W::Escape => V::Escape,
        W::F1 => V::F1,
        W::F2 => V::F2,
        W::F3 => V::F3,
        W::F4 => V::F4,
        W::F5 => V::F5,
        W::F6 => V::F6,
        W::F7 => V::F7,
        W::F8 => V::F8,
        W::F9 => V::F9,
        W::F10 => V::F10,
        W::F11 => V::F11,
        W::F12 => V::F12,
        W::PrintScreen => V::Snapshot,
        W::ScrollLock => V::Scroll,
        W::Pause => V::Pause,
        W::Insert => V::Insert,
        W::Home => V::Home,
        W::Delete => V::Delete,
        W::End => V::End,
        W::PageDown => V::PageDown,
        W::PageUp => V::PageUp,
        W::NumLock => V::Numlock,
        W::Numpad0 => V::Numpad0,
        W::Numpad1 => V::Numpad1,
        W::Numpad2 => V::Numpad2,
        W::Numpad3 => V::Numpad3,
        W::Numpad4 => V::Numpad4,
        W::Numpad5 => V::Numpad5,
        W::Numpad6 => V::Numpad6,
        W::Numpad7 => V::Numpad7,
        W::Numpad8 => V::Numpad8,
        W::Numpad9 => V::Numpad9,
        W::NumpadDecimal => V::Decimal,
        W::NumpadDivide => V::Divide,
        W::NumpadMultiply => V::Multiply,
        W::NumpadSubtract => V::Subtract,
        W::NumpadAdd => V::Add,
        W::NumpadEnter => V::NumpadEnter,
        W::NumpadEqual => V::NumpadEquals,
        W::NumpadComma => V::NumpadComma,
        W::ArrowLeft => V::Left,
        W::ArrowUp => V::Up,
        W::ArrowRight => V::Right,
        W::ArrowDown => V::Down,
        W::Backspace => V::Back,
        W::Enter => V::Return,
        W::Space => V::Space,
        W::Tab => V::Tab,
        W::ShiftLeft => V::LShift,
        W::ShiftRight => V::RShift,
        W::ControlLeft => V::LControl,
        W::ControlRight => V::RControl,
        W::AltLeft => V::LAlt,
        W::AltRight => V::RAlt,
        W::BracketLeft => V::LBracket,
        W::BracketRight => V::RBracket,
        W::Minus => V::Minus,
        W::Equal => V::Equals,
        W::Comma => V::Comma,
        W::Period => V::Period,
        W::Semicolon => V::Semicolon,
        W::Slash => V::Slash,
        W::Backslash => V::Backslash,
        W::Quote => V::Apostrophe,
        W::Backquote => V::Grave,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::map_keycode;
    use crate::hal::VirtualKeyCode;
    use winit::keyboard::KeyCode;

    #[test]
    fn maps_representative_keys() {
        assert_eq!(map_keycode(KeyCode::KeyA), Some(VirtualKeyCode::A));
        assert_eq!(map_keycode(KeyCode::ArrowLeft), Some(VirtualKeyCode::Left));
        assert_eq!(map_keycode(KeyCode::Numpad8), Some(VirtualKeyCode::Numpad8));
        assert_eq!(
            map_keycode(KeyCode::PrintScreen),
            Some(VirtualKeyCode::Snapshot)
        );
        assert_eq!(
            map_keycode(KeyCode::NumpadEnter),
            Some(VirtualKeyCode::NumpadEnter)
        );
    }

    #[test]
    fn returns_none_for_unmapped_keys() {
        assert_eq!(map_keycode(KeyCode::SuperLeft), None);
    }
}

pub fn main_loop<GS: GameState>(mut bterm: BTerm, mut gamestate: GS) -> BResult<()> {
    let now = Instant::now();
    let mut prev_seconds = now.elapsed().as_secs();
    let mut prev_ms = now.elapsed().as_millis();
    let mut frames = 0;

    let mut backing_flip: QuadRender = {
        let be = BACKEND.lock();
        let wgpu = be.wgpu.as_ref().unwrap();
        let mut bit = BACKEND_INTERNAL.lock();
        for f in bit.fonts.iter_mut() {
            f.setup_wgpu_texture(wgpu)?;
        }

        for s in bit.sprite_sheets.iter_mut() {
            let mut f = Font::new(&s.filename.to_string(), 1, 1, (1, 1));
            f.setup_wgpu_texture(wgpu)?;
            s.backing = Some(Rc::new(Box::new(f)));
        }

        QuadRender::new(wgpu, &bit.shaders[2])
    };

    // We're doing a little dance here to get around lifetime/borrow checking.
    // Removing the context data from BTerm in an atomic swap, so it isn't borrowed after move.
    let wrap = { std::mem::replace(&mut BACKEND.lock().context_wrapper, None) };
    let unwrap = wrap.unwrap();

    let el = unwrap.el;
    let window = unwrap.window;

    on_resize(
        &mut bterm,
        window.inner_size(),
        window.scale_factor(),
        true,
        &mut backing_flip,
    )?; // Additional resize to handle some X11 cases

    let mut queued_resize_event: Option<ResizeEvent> = None;
    #[cfg(feature = "low_cpu")]
    let spin_sleeper = spin_sleep::SpinSleeper::default();
    let my_window_id = window.id();

    #[allow(deprecated)]
    el.run(move |event, target| {
        let wait_time = BACKEND.lock().frame_sleep_time.unwrap_or(33); // Hoisted to reduce locks
        target.set_control_flow(TICK_TYPE);

        if bterm.quitting {
            target.exit();
            return;
        }

        match event {
            Event::AboutToWait => {
                let frame_timer = Instant::now();
                if window.inner_size().width == 0 || window.inner_size().height == 0 {
                    return;
                }

                let execute_ms = now.elapsed().as_millis() as u64 - prev_ms as u64;
                if execute_ms >= wait_time {
                    if queued_resize_event.is_some() {
                        if let Some(resize) = &queued_resize_event {
                            on_resize(
                                &mut bterm,
                                resize.physical_size,
                                resize.dpi_scale_factor,
                                resize.send_event,
                                &mut backing_flip,
                            )
                            .unwrap();
                        }
                        queued_resize_event = None;
                    }

                    tock(
                        &mut bterm,
                        &mut gamestate,
                        &mut frames,
                        &mut prev_seconds,
                        &mut prev_ms,
                        &now,
                        &mut backing_flip,
                    );
                    clear_input_state(&mut bterm);
                }

                // Wait for an appropriate amount of time
                let time_since_last_frame = frame_timer.elapsed().as_millis() as u64;
                if time_since_last_frame < wait_time {
                    #[cfg(feature = "low_cpu")]
                    let delay = u64::min(33, wait_time - time_since_last_frame);
                    #[cfg(feature = "low_cpu")]
                    spin_sleeper.sleep(std::time::Duration::from_millis(delay));
                }
            }
            Event::WindowEvent { event, window_id } => {
                if window_id != my_window_id {
                    return;
                }

                match event {
                    WindowEvent::Moved(physical_position) => {
                        bterm.on_event(BEvent::Moved {
                            new_position: Point::new(physical_position.x, physical_position.y),
                        });

                        let scale_factor = window.scale_factor();
                        let physical_size = window.inner_size();
                        queued_resize_event = Some(ResizeEvent {
                            physical_size,
                            dpi_scale_factor: scale_factor,
                            send_event: true,
                        });
                    }
                    WindowEvent::Resized(_physical_size) => {
                        let scale_factor = window.scale_factor();
                        let physical_size = window.inner_size();
                        queued_resize_event = Some(ResizeEvent {
                            physical_size,
                            dpi_scale_factor: scale_factor,
                            send_event: true,
                        });
                    }
                    WindowEvent::CloseRequested => {
                        if !INPUT.lock().use_events {
                            target.exit();
                        } else {
                            bterm.on_event(BEvent::CloseRequested);
                        }
                    }
                    WindowEvent::Focused(focused) => {
                        bterm.on_event(BEvent::Focused { focused });
                    }
                    WindowEvent::CursorMoved { position: pos, .. } => {
                        bterm.on_mouse_position(pos.x, pos.y);
                    }
                    WindowEvent::CursorEntered { .. } => bterm.on_event(BEvent::CursorEntered),
                    WindowEvent::CursorLeft { .. } => bterm.on_event(BEvent::CursorLeft),

                    WindowEvent::MouseInput { button, state, .. } => {
                        let button = match &button {
                            MouseButton::Left => 0,
                            MouseButton::Right => 1,
                            MouseButton::Middle => 2,
                            MouseButton::Back => 3,
                            MouseButton::Forward => 4,
                            MouseButton::Other(num) => 5 + *num as usize,
                        };
                        bterm.on_mouse_button(button, state == ElementState::Pressed);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        let scale_factor = window.scale_factor();
                        let physical_size = window.inner_size();
                        on_resize(
                            &mut bterm,
                            physical_size,
                            scale_factor,
                            false,
                            &mut backing_flip,
                        )
                        .unwrap();
                        bterm.on_event(BEvent::ScaleFactorChanged {
                            new_size: Point::new(physical_size.width, physical_size.height),
                            dpi_scale_factor: scale_factor as f32,
                        })
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        if let Some(text) = event.text {
                            for c in text.chars() {
                                bterm.on_event(BEvent::Character { c });
                            }
                        }

                        if let PhysicalKey::Code(code) = event.physical_key {
                            if let Some(key) = map_keycode(code) {
                                bterm.on_key(key, key as u32, event.state == ElementState::Pressed);
                            }
                        }
                    }

                    WindowEvent::ModifiersChanged(modifiers) => {
                        let state = modifiers.state();
                        bterm.shift = state.shift_key();
                        bterm.alt = state.alt_key();
                        bterm.control = state.control_key();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

fn largest_active_font() -> (u32, u32) {
    let bi = BACKEND_INTERNAL.lock();
    let mut max_width = 0;
    let mut max_height = 0;
    bi.consoles.iter().for_each(|c| {
        let size = bi.fonts[c.font_index].tile_size;
        if size.0 > max_width {
            max_width = size.0;
        }
        if size.1 > max_height {
            max_height = size.1;
        }
    });
    (max_width, max_height)
}

fn on_resize(
    bterm: &mut BTerm,
    physical_size: PhysicalSize<u32>,
    dpi_scale_factor: f64,
    send_event: bool,
    backing_flip: &mut QuadRender,
) -> BResult<()> {
    let font_max_size = largest_active_font();
    //println!("{:#?}", physical_size);
    INPUT.lock().set_scale_factor(dpi_scale_factor);
    let mut be = BACKEND.lock();
    let (l, r, t, b) = be.screen_scaler.get_backing_buffer_output_coordinates();
    be.screen_scaler.change_physical_size_smooth(
        physical_size.width,
        physical_size.height,
        dpi_scale_factor as f32,
        font_max_size,
    );
    if send_event {
        bterm.resize_pixels(
            physical_size.width as u32,
            physical_size.height as u32,
            be.resize_scaling,
        );
    }

    // WGPU resizing
    if let Some(wgpu) = be.wgpu.as_mut() {
        backing_flip.update_buffer_with_gutter(wgpu, l, r, t, b);
        wgpu.config.width = physical_size.width;
        wgpu.config.height = physical_size.height;
        wgpu.surface.configure(&wgpu.device, &wgpu.config);
    }

    // Messaging
    bterm.on_event(BEvent::Resized {
        new_size: Point::new(
            be.screen_scaler.available_width,
            be.screen_scaler.available_height,
        ),
        dpi_scale_factor: dpi_scale_factor as f32,
    });

    // Consoles
    let mut bit = BACKEND_INTERNAL.lock();
    if be.resize_scaling && send_event {
        // Backing buffer resizing
        let w = be.screen_scaler.available_width;
        let h = be.screen_scaler.available_height;
        let wgpu = be.wgpu.as_mut().unwrap();
        wgpu.backing_buffer = Framebuffer::new(&wgpu.device, wgpu.config.format, w, h);

        let num_consoles = bit.consoles.len();
        for i in 0..num_consoles {
            let font_size = bit.fonts[bit.consoles[i].font_index].tile_size;
            let chr_w = be.screen_scaler.available_width / font_size.0;
            let chr_h = be.screen_scaler.available_height / font_size.1;
            bit.consoles[i].console.set_char_size(chr_w, chr_h);
        }
    }

    Ok(())
}

/// Internal handling of the main loop.
fn tock<GS: GameState>(
    bterm: &mut BTerm,
    gamestate: &mut GS,
    frames: &mut i32,
    prev_seconds: &mut u64,
    prev_ms: &mut u128,
    now: &Instant,
    backing_flip: &mut QuadRender,
) {
    // Check that the console backings match our actual consoles
    check_console_backing();

    let now_seconds = now.elapsed().as_secs();
    *frames += 1;

    if now_seconds > *prev_seconds {
        bterm.fps = *frames as f32 / (now_seconds - *prev_seconds) as f32;
        *frames = 0;
        *prev_seconds = now_seconds;
    }

    let now_ms = now.elapsed().as_millis();
    if now_ms > *prev_ms {
        bterm.frame_time_ms = (now_ms - *prev_ms) as f32;
        *prev_ms = now_ms;
    }

    // Console structure - doesn't really have to be every frame...
    rebuild_consoles();

    // Run the main loop
    gamestate.tick(bterm);

    // Tell each console to draw itself
    render_consoles().unwrap();

    // If there is a GL callback, call it now
    /*{
        let be = BACKEND.lock();
        if let Some(callback) = be.gl_callback.as_ref() {
            let gl = be.gl.as_ref().unwrap();
            callback(gamestate, gl);
        }
    }*/

    // Present the output now that we've done all the layers and
    // backing buffer/post-process
    {
        let mut be = BACKEND.lock();
        let screenshot_request = be.request_screenshot.clone();
        let mut clear_screenshot_request = false;
        if let Some(wgpu) = be.wgpu.as_ref() {
            let (current_tex, reconfigure) = match wgpu.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => (Some(t), false),
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => (Some(t), true),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    wgpu.surface.configure(&wgpu.device, &wgpu.config);
                    (None, false)
                }
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Validation => (None, false),
            };

            if let Some(current_tex) = current_tex {
                backing_flip.update_uniform(
                    wgpu,
                    bterm.post_scanlines,
                    bterm.post_screenburn,
                    bterm.screen_burn_color,
                );
                let target = current_tex
                    .texture
                    .create_view(&TextureViewDescriptor::default());
                if backing_flip.render(wgpu, &target).is_ok() {
                    if let Some(filename) = &screenshot_request {
                        take_screenshot(filename, wgpu, bterm, &wgpu.backing_buffer.texture);
                    }
                    clear_screenshot_request = true;
                    current_tex.present();
                }
                if reconfigure {
                    wgpu.surface.configure(&wgpu.device, &wgpu.config);
                }
            }
        }
        if clear_screenshot_request {
            be.request_screenshot = None;
        }
    }
}

pub(crate) fn rebuild_consoles() {
    let must_resize = BACKEND.lock().screen_scaler.get_resized_and_reset();
    let mut consoles = CONSOLE_BACKING.lock();
    let mut bi = BACKEND_INTERNAL.lock();
    //let ss = bi.sprite_sheets.clone();
    for (i, c) in consoles.iter_mut().enumerate() {
        let font_index = bi.consoles[i].font_index;
        let glyph_dimensions = bi.fonts[font_index].font_dimensions_glyphs;
        let tex_dimensions = bi.fonts[font_index].font_dimensions_texture;
        let cons = &mut bi.consoles[i];
        match c {
            ConsoleBacking::Simple { backing } => {
                let sc = cons
                    .console
                    .as_any_mut()
                    .downcast_mut::<SimpleConsole>()
                    .unwrap();
                if sc.is_dirty {
                    let be = BACKEND.lock();
                    let wgpu = be.wgpu.as_ref().unwrap();
                    backing.rebuild_vertices(
                        wgpu,
                        sc.height,
                        sc.width,
                        &sc.tiles,
                        sc.offset_x,
                        sc.offset_y,
                        sc.scale,
                        sc.scale_center,
                        sc.needs_resize_internal || must_resize,
                        FontScaler::new(glyph_dimensions, tex_dimensions),
                        &be.screen_scaler,
                    );
                    sc.needs_resize_internal = false;
                }
            }
            ConsoleBacking::Sparse { backing } => {
                let sc = bi.consoles[i]
                    .console
                    .as_any_mut()
                    .downcast_mut::<SparseConsole>()
                    .unwrap();
                if sc.is_dirty {
                    let be = BACKEND.lock();
                    let wgpu = be.wgpu.as_ref().unwrap();
                    backing.rebuild_vertices(
                        wgpu,
                        sc.height,
                        sc.width,
                        sc.offset_x,
                        sc.offset_y,
                        sc.scale,
                        sc.scale_center,
                        &sc.tiles,
                        FontScaler::new(glyph_dimensions, tex_dimensions),
                        &be.screen_scaler,
                        must_resize,
                    );
                    sc.needs_resize_internal = false;
                }
            }
            ConsoleBacking::Fancy { backing } => {
                let fc = bi.consoles[i]
                    .console
                    .as_any_mut()
                    .downcast_mut::<FlexiConsole>()
                    .unwrap();
                if fc.is_dirty {
                    let be = BACKEND.lock();
                    let wgpu = be.wgpu.as_ref().unwrap();
                    fc.tiles.sort_by(|a, b| a.z_order.cmp(&b.z_order));
                    backing.rebuild_vertices(
                        wgpu,
                        fc.height,
                        fc.width,
                        fc.offset_x,
                        fc.offset_y,
                        fc.scale,
                        fc.scale_center,
                        &fc.tiles,
                        FontScaler::new(glyph_dimensions, tex_dimensions),
                        &be.screen_scaler,
                    );
                    fc.needs_resize_internal = false;
                }
            }
            ConsoleBacking::Sprite { backing } => {
                let ss = bi.sprite_sheets.clone();
                let sc = bi.consoles[i]
                    .console
                    .as_any_mut()
                    .downcast_mut::<SpriteConsole>()
                    .unwrap();

                if sc.is_dirty {
                    let be = BACKEND.lock();
                    let wgpu = be.wgpu.as_ref().unwrap();
                    sc.sprites.sort_by(|a, b| a.z_order.cmp(&b.z_order));
                    backing.rebuild_vertices(
                        wgpu,
                        sc.height,
                        sc.width,
                        &sc.sprites,
                        &ss[sc.sprite_sheet],
                    );
                    sc.needs_resize_internal = false;
                }
            }
        }
    }
}

pub(crate) fn render_consoles() -> BResult<()> {
    let bi = BACKEND_INTERNAL.lock();
    let mut consoles = CONSOLE_BACKING.lock();
    //let output = BACKEND.lock().backing_buffer.as_ref().unwrap().view();
    clear_screen_pass()?;
    for (i, c) in consoles.iter_mut().enumerate() {
        let cons = &bi.consoles[i];
        let font = &bi.fonts[cons.font_index];
        match c {
            ConsoleBacking::Simple { backing } => {
                backing.wgpu_draw(font)?;
            }
            ConsoleBacking::Sparse { backing } => {
                backing.wgpu_draw(font)?;
            }
            ConsoleBacking::Fancy { backing } => {
                backing.wgpu_draw(font)?;
            }
            ConsoleBacking::Sprite { backing } => {
                backing.wgpu_draw(&bi.sprite_sheets[0].backing.as_ref().unwrap())?;
            }
        }
    }
    Ok(())
}

pub(crate) fn check_console_backing() {
    let be = BACKEND.lock();
    let mut consoles = CONSOLE_BACKING.lock();
    if consoles.is_empty() {
        // Easy case: there are no consoles so we need to make them all.
        let bit = BACKEND_INTERNAL.lock();
        for cons in &bit.consoles {
            let cons_any = cons.console.as_any();
            if let Some(st) = cons_any.downcast_ref::<SimpleConsole>() {
                consoles.push(ConsoleBacking::Simple {
                    backing: SimpleConsoleBackend::new(
                        st.width as usize,
                        st.height as usize,
                        be.wgpu.as_ref().unwrap(),
                        &bit.shaders[cons.shader_index],
                        &bit.fonts[cons.font_index],
                    ),
                });
            } else if let Some(_sp) = cons_any.downcast_ref::<SparseConsole>() {
                consoles.push(ConsoleBacking::Sparse {
                    backing: SparseConsoleBackend::new(
                        be.wgpu.as_ref().unwrap(),
                        &bit.shaders[cons.shader_index],
                        &bit.fonts[cons.font_index],
                    ),
                });
            } else if let Some(_sp) = cons_any.downcast_ref::<FlexiConsole>() {
                consoles.push(ConsoleBacking::Fancy {
                    backing: FancyConsoleBackend::new(
                        be.wgpu.as_ref().unwrap(),
                        &bit.shaders[3],
                        &bit.fonts[cons.font_index],
                    ),
                });
            } else if let Some(_sp) = cons_any.downcast_ref::<SpriteConsole>() {
                //let bi = BACKEND_INTERNAL.lock();
                consoles.push(ConsoleBacking::Sprite {
                    backing: SpriteConsoleBackend::new(
                        be.wgpu.as_ref().unwrap(),
                        &bit.shaders[4],
                        &bit.sprite_sheets[0].backing.as_ref().unwrap(),
                    ),
                });
            } else {
                panic!("Unknown console type.");
            }
        }
    }
}

fn clear_screen_pass() -> BResult<()> {
    let mut be = BACKEND.lock();
    if let Some(wgpu) = be.wgpu.as_mut() {
        let mut encoder = wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: wgpu.backing_buffer.view(),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        // submit will accept anything that implements IntoIter
        wgpu.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    } else {
        Err("WebGPU backend not initialized".into())
    }
}

fn take_screenshot(_filename: &str, wgpu: &WgpuLink, bterm: &BTerm, texture: &wgpu::Texture) {
    let w = (bterm.width_pixels as f32) as usize;
    let h = (bterm.height_pixels as f32) as usize;
    //println!("Taking screenshot {} = {}x{}", filename, w, h);
    let buffer_dimensions = BufferDimensions::new(w, h);
    let output_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let texture_extent = wgpu::Extent3d {
        width: buffer_dimensions.width as u32,
        height: buffer_dimensions.height as u32,
        depth_or_array_layers: 1,
    };

    let mut encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    //println!("Copying texture to buffer");
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                rows_per_image: None,
            },
        },
        texture_extent,
    );
    wgpu.queue.submit(std::iter::once(encoder.finish()));
    //println!("Saving PNG");

    // This has apparently changed a lot - rewrite
    /*let buffer_slice = output_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read, move |buffer_slice| {
        if let Ok(buffer_slice) = buffer_slice {
            let padded_buffer = buffer_slice.get_mapped_range();
            let mut png_encoder = png::Encoder::new(
                File::create(filename).unwrap(),
                buffer_dimensions.width as u32,
                buffer_dimensions.height as u32,
            );
            png_encoder.set_depth(png::BitDepth::Eight);
            png_encoder.set_color(png::ColorType::RGBA);
            let mut png_writer = png_encoder
                .write_header()
                .unwrap()
                .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row);

            // from the padded_buffer we write just the unpadded bytes into the image
            for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
                png_writer
                    .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                    .unwrap();
            }
            png_writer.finish().unwrap();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            //println!("Unmapping");
            std::mem::drop(padded_buffer);
            output_buffer.unmap();
        }
    });*/
}

struct BufferDimensions {
    width: usize,
    height: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            padded_bytes_per_row,
        }
    }
}
