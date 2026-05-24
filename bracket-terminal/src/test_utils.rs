use super::console::{CharacterTranslationMode, Console, TextAlign};
use crate::prelude::FontCharType;
use bracket_color::prelude::RGBA;
use bracket_geometry::prelude::Rect;
use bracket_rex::prelude::XpLayer;
use std::any::Any;

pub(crate) struct TestConsole {
    pub size: (u32, u32),
    pub clipping: Option<Rect>,
}

impl TestConsole {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            size: (width, height),
            clipping: None,
        }
    }

    pub(crate) fn with_clipping(mut self, clipping: Rect) -> Self {
        self.clipping = Some(clipping);
        self
    }
}

impl Console for TestConsole {
    fn get_char_size(&self) -> (u32, u32) {
        self.size
    }

    fn at(&self, x: i32, y: i32) -> usize {
        y as usize * self.size.0 as usize + x as usize
    }

    fn get_clipping(&self) -> Option<Rect> {
        self.clipping
    }

    fn set_clipping(&mut self, clipping: Option<Rect>) {
        self.clipping = clipping;
    }

    fn resize_pixels(&mut self, _width: u32, _height: u32) {}
    fn cls(&mut self) {}
    fn cls_bg(&mut self, _background: RGBA) {}
    fn print(&mut self, _x: i32, _y: i32, _output: &str) {}
    fn print_color(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _output: &str) {}
    fn printer(
        &mut self,
        _x: i32,
        _y: i32,
        _output: &str,
        _align: TextAlign,
        _background: Option<RGBA>,
    ) {
    }
    fn set(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _glyph: FontCharType) {}
    fn set_bg(&mut self, _x: i32, _y: i32, _bg: RGBA) {}
    fn draw_box(&mut self, _x: i32, _y: i32, _width: i32, _height: i32, _fg: RGBA, _bg: RGBA) {}
    fn draw_hollow_box(
        &mut self,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
    }
    fn draw_box_double(
        &mut self,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
    }
    fn draw_hollow_box_double(
        &mut self,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
    }
    fn fill_region(&mut self, _target: Rect, _glyph: FontCharType, _fg: RGBA, _bg: RGBA) {}
    fn draw_bar_horizontal(
        &mut self,
        _x: i32,
        _y: i32,
        _width: i32,
        _n: i32,
        _max: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
    }
    fn draw_bar_vertical(
        &mut self,
        _x: i32,
        _y: i32,
        _height: i32,
        _n: i32,
        _max: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
    }
    fn print_centered(&mut self, _y: i32, _text: &str) {}
    fn print_color_centered(&mut self, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {}
    fn print_centered_at(&mut self, _x: i32, _y: i32, _text: &str) {}
    fn print_color_centered_at(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {}
    fn print_right(&mut self, _x: i32, _y: i32, _text: &str) {}
    fn print_color_right(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {}

    fn to_xp_layer(&self) -> XpLayer {
        XpLayer::new(self.size.0 as usize, self.size.1 as usize)
    }

    fn set_offset(&mut self, _x: f32, _y: f32) {}
    fn set_scale(&mut self, _scale: f32, _center_x: i32, _center_y: i32) {}
    fn get_scale(&self) -> (f32, i32, i32) {
        (1.0, 0, 0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_all_fg_alpha(&mut self, _alpha: f32) {}
    fn set_all_bg_alpha(&mut self, _alpha: f32) {}
    fn set_all_alpha(&mut self, _fg: f32, _bg: f32) {}
    fn set_translation_mode(&mut self, _mode: CharacterTranslationMode) {}
    fn set_char_size(&mut self, _width: u32, _height: u32) {}
    fn clear_dirty(&mut self) {}
}
