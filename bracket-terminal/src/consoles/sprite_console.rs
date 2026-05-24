use crate::prelude::{CharacterTranslationMode, Console, FontCharType, TextAlign};
use bracket_color::prelude::RGBA;
use bracket_geometry::prelude::Rect;
use bracket_rex::prelude::XpLayer;
use std::any::Any;

/// Internal storage structure for sparse tiles.
pub struct RenderSprite {
    pub destination: Rect,
    pub z_order: i32,
    pub tint: RGBA,
    pub index: usize,
}

/// A sparse console. Rather than storing every cell on the screen, it stores just cells that have
/// data.
pub struct SpriteConsole {
    pub width: u32,
    pub height: u32,
    pub sprite_sheet: usize,

    pub sprites: Vec<RenderSprite>,
    pub is_dirty: bool,

    pub(crate) needs_resize_internal: bool,
}

impl SpriteConsole {
    /// Initializes the console.
    pub fn init(width: u32, height: u32, sprite_sheet: usize) -> Box<SpriteConsole> {
        // Console backing initialization
        let new_console = SpriteConsole {
            width,
            height,
            sprites: Vec::new(),
            is_dirty: true,
            needs_resize_internal: false,
            sprite_sheet,
        };

        Box::new(new_console)
    }

    pub fn render_sprite(&mut self, sprite: RenderSprite) {
        self.sprites.push(sprite);
        self.is_dirty = true;
    }
}

impl Console for SpriteConsole {
    fn get_char_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize_pixels(&mut self, _width: u32, _height: u32) {
        self.is_dirty = true;
    }

    /// Translates x/y to an index entry. Not really useful.
    fn at(&self, x: i32, y: i32) -> usize {
        (((self.height - 1 - y as u32) * self.width) + x as u32) as usize
    }

    /// Clear the screen.
    fn cls(&mut self) {
        self.is_dirty = true;
        self.sprites.clear();
    }

    /// Clear the screen. Since we don't HAVE a background, it doesn't use it.
    fn cls_bg(&mut self, _background: RGBA) {
        self.is_dirty = true;
        self.sprites.clear();
    }

    /// Prints a string to an x/y position.
    fn print(&mut self, _x: i32, _y: i32, _output: &str) {
        // Does nothing
    }

    /// Prints a string to an x/y position, with foreground and background colors.
    fn print_color(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _output: &str) {
        // Does nothing
    }

    /// Sets a single cell in the console
    fn set(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _glyph: FontCharType) {
        // Does nothing
    }

    /// Sets a single cell in the console's background
    fn set_bg(&mut self, _x: i32, _y: i32, _bg: RGBA) {
        // Does nothing for this layer type
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_box(&mut self, _sx: i32, _sy: i32, _width: i32, _height: i32, _fg: RGBA, _bg: RGBA) {
        // Does nothing
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 double line characters
    fn draw_box_double(
        &mut self,
        _sx: i32,
        _sy: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
        // Does nothing
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_hollow_box(
        &mut self,
        _sx: i32,
        _sy: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
        // Does nothing
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 double line characters
    fn draw_hollow_box_double(
        &mut self,
        _sx: i32,
        _sy: i32,
        _width: i32,
        _height: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
        // Does nothing
    }

    /// Fills a rectangle with the specified rendering information
    fn fill_region(&mut self, _target: Rect, _glyph: FontCharType, _fg: RGBA, _bg: RGBA) {
        // Does nothing
    }

    /// Draws a horizontal progress bar
    fn draw_bar_horizontal(
        &mut self,
        _sx: i32,
        _sy: i32,
        _width: i32,
        _n: i32,
        _max: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
        // Does nothing
    }

    /// Draws a vertical progress bar
    fn draw_bar_vertical(
        &mut self,
        _sx: i32,
        _sy: i32,
        _height: i32,
        _n: i32,
        _max: i32,
        _fg: RGBA,
        _bg: RGBA,
    ) {
        // Does nothing
    }

    /// Prints text, centered to the whole console width, at vertical location y.
    fn print_centered(&mut self, _y: i32, _text: &str) {
        // Does nothing
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered(&mut self, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {
        // Does nothing
    }

    /// Prints text, centered to the whole console width, at vertical location y.
    fn print_centered_at(&mut self, _x: i32, _y: i32, _text: &str) {
        // Does nothing
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered_at(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {
        // Does nothing
    }

    /// Prints text right-aligned
    fn print_right(&mut self, _x: i32, _y: i32, _text: &str) {
        // Does nothing
    }

    /// Prints colored text right-aligned
    fn print_color_right(&mut self, _x: i32, _y: i32, _fg: RGBA, _bg: RGBA, _text: &str) {
        // Does nothing
    }

    /// Print a colorized string with the color encoding defined inline.
    /// For example: printer(1, 1, "#[blue]This blue text contains a #[pink]pink#[] word")
    /// You can get the same effect with a TextBlock, but this can be easier.
    /// Thanks to doryen_rs for the idea.
    fn printer(
        &mut self,
        _x: i32,
        _y: i32,
        _output: &str,
        _align: TextAlign,
        _background: Option<RGBA>,
    ) {
        // Does nothing
    }

    /// Saves the layer to an XpFile structure
    fn to_xp_layer(&self) -> XpLayer {
        XpLayer::new(self.width as usize, self.height as usize)
    }

    /// Sets an offset to total console rendering, useful for layers that
    /// draw between tiles. Offsets are specified as a percentage of total
    /// character size; so -0.5 will offset half a character to the left/top.
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

    /// Permits the creation of an arbitrary clipping rectangle. It's a really good idea
    /// to make sure that this rectangle is entirely valid.
    fn set_clipping(&mut self, _clipping: Option<Rect>) {}

    /// Returns the current arbitrary clipping rectangle, None if there isn't one.
    fn get_clipping(&self) -> Option<Rect> {
        None
    }

    /// Sets ALL tiles foreground alpha (only tiles that exist, in sparse consoles).
    fn set_all_fg_alpha(&mut self, alpha: f32) {
        self.sprites.iter_mut().for_each(|t| t.tint.a = alpha);
    }

    /// Sets ALL tiles background alpha (only tiles that exist, in sparse consoles).
    fn set_all_bg_alpha(&mut self, _alpha: f32) {}

    /// Sets ALL tiles foreground alpha (only tiles that exist, in sparse consoles).
    fn set_all_alpha(&mut self, fg: f32, _bg: f32) {
        self.sprites.iter_mut().for_each(|t| {
            t.tint.a = fg;
        });
    }

    /// Sets the character translation mode
    fn set_translation_mode(&mut self, _mode: CharacterTranslationMode) {}

    /// Sets the character size of the terminal
    fn set_char_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.needs_resize_internal = true;
    }

    // Clears the dirty bit
    fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn rgba(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        RGBA::from_u8(r, g, b, a)
    }

    fn test_sprite(index: usize) -> RenderSprite {
        RenderSprite {
            destination: Rect::with_size(1, 2, 3, 4),
            z_order: 5,
            tint: rgba(10, 20, 30, 40),
            index,
        }
    }

    #[test]
    fn init_creates_empty_sprite_console() {
        let console = SpriteConsole::init(80, 50, 7);

        assert_eq!(console.get_char_size(), (80, 50));
        assert_eq!(console.sprite_sheet, 7);
        assert!(console.sprites.is_empty());
        assert!(console.is_dirty);
        assert!(!console.needs_resize_internal);
    }

    #[rstest]
    #[case(0, 0, 30)]
    #[case(1, 0, 31)]
    #[case(9, 0, 39)]
    #[case(0, 1, 20)]
    #[case(0, 2, 10)]
    #[case(0, 3, 0)]
    #[case(9, 3, 9)]
    fn at_uses_bottom_origin_storage(#[case] x: i32, #[case] y: i32, #[case] expected: usize) {
        let console = SpriteConsole::init(10, 4, 0);
        assert_eq!(console.at(x, y), expected);
    }

    #[test]
    fn sprite_console_at_mapping_differs_from_test_console_row_major_mapping() {
        let sprite_console = SpriteConsole::init(10, 4, 0);

        assert_eq!(sprite_console.at(0, 0), 30);
        assert_eq!(sprite_console.at(0, 3), 0);
    }

    #[test]
    fn render_sprite_appends_sprite_and_marks_dirty() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.clear_dirty();
        console.render_sprite(test_sprite(3));

        assert!(console.is_dirty);
        assert_eq!(console.sprites.len(), 1);
        assert_eq!(console.sprites[0].destination, Rect::with_size(1, 2, 3, 4));
        assert_eq!(console.sprites[0].z_order, 5);
        assert_eq!(console.sprites[0].tint, rgba(10, 20, 30, 40));
        assert_eq!(console.sprites[0].index, 3);
    }

    #[test]
    fn cls_clears_sprites_and_marks_dirty() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        console.render_sprite(test_sprite(2));
        console.clear_dirty();
        assert!(!console.is_dirty);

        console.cls();
        assert!(console.is_dirty);
        assert!(console.sprites.is_empty());
    }

    #[test]
    fn cls_bg_clears_sprites_and_ignores_background() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        console.clear_dirty();
        assert!(!console.is_dirty);

        console.cls_bg(rgba(1, 2, 3, 4));
        assert!(console.is_dirty);
        assert!(console.sprites.is_empty());
    }

    #[test]
    fn text_and_tile_methods_do_nothing() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.print(1, 2, "hello");
        console.print_color(1, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), "hello");
        console.set(1, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 65);
        console.set_bg(1, 2, rgba(1, 2, 3, 4));
        console.fill_region(
            Rect::with_size(0, 0, 2, 2),
            65,
            rgba(1, 2, 3, 4),
            rgba(5, 6, 7, 8),
        );
        console.print_centered(1, "hello");
        console.print_color_centered(1, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), "hello");
        console.print_centered_at(1, 2, "hello");
        console.print_color_centered_at(1, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), "hello");
        console.print_right(10, 2, "hello");
        console.print_color_right(10, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), "hello");
        console.printer(1, 2, "#[red]hello", TextAlign::Left, None);

        assert!(console.sprites.is_empty());
    }

    #[test]
    fn draw_methods_do_nothing() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.draw_box(1, 2, 3, 4, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));
        console.draw_box_double(1, 2, 3, 4, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));
        console.draw_hollow_box(1, 2, 3, 4, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));
        console.draw_hollow_box_double(1, 2, 3, 4, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));
        console.draw_bar_horizontal(1, 2, 3, 1, 10, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));
        console.draw_bar_vertical(1, 2, 3, 1, 10, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8));

        assert!(console.sprites.is_empty());
    }

    #[test]
    fn to_xp_layer_returns_layer_with_console_dimensions() {
        let console = SpriteConsole::init(80, 50, 7);
        let layer = console.to_xp_layer();

        assert_eq!(layer.width, 80);
        assert_eq!(layer.height, 50);
    }

    #[test]
    fn scale_methods_are_no_ops_with_fixed_default_result() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.set_offset(1.0, 2.0);
        console.set_scale(3.0, 4, 5);

        assert_eq!(console.get_scale(), (1.0, 0, 0));
    }

    #[test]
    fn clipping_methods_are_no_ops() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.set_clipping(Some(Rect::with_size(1, 2, 3, 4)));
        assert_eq!(console.get_clipping(), None);
    }

    #[test]
    fn set_all_fg_alpha_updates_sprite_tint_alpha() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        console.render_sprite(test_sprite(2));
        console.set_all_fg_alpha(0.25);

        assert!(console.sprites.iter().all(|sprite| sprite.tint.a == 0.25));
    }

    #[test]
    fn set_all_bg_alpha_does_nothing() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        let original_alpha = console.sprites[0].tint.a;

        console.set_all_bg_alpha(0.25);
        assert_eq!(console.sprites[0].tint.a, original_alpha);
    }

    #[test]
    fn set_all_alpha_updates_sprite_tint_alpha_from_fg_only() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        console.render_sprite(test_sprite(2));
        console.set_all_alpha(0.75, 0.25);

        assert!(console.sprites.iter().all(|sprite| sprite.tint.a == 0.75));
    }

    #[test]
    fn set_translation_mode_does_nothing() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.set_translation_mode(CharacterTranslationMode::Unicode);
        assert!(console.sprites.is_empty());
    }

    #[test]
    fn set_char_size_updates_dimensions_without_changing_sprites() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.render_sprite(test_sprite(1));
        console.set_char_size(100, 60);

        assert_eq!(console.get_char_size(), (100, 60));
        assert_eq!(console.sprites.len(), 1);
        assert!(console.needs_resize_internal);
    }

    #[test]
    fn resize_pixels_marks_dirty_without_changing_dimensions() {
        let mut console = SpriteConsole::init(80, 50, 7);

        console.clear_dirty();
        console.resize_pixels(1024, 768);

        assert!(console.is_dirty);
        assert_eq!(console.get_char_size(), (80, 50));
    }

    #[test]
    fn clear_dirty_resets_dirty_flag() {
        let mut console = SpriteConsole::init(80, 50, 7);
        assert!(console.is_dirty);

        console.clear_dirty();
        assert!(!console.is_dirty);
    }

    #[test]
    fn as_any_allows_downcasting_to_sprite_console() {
        let console = SpriteConsole::init(80, 50, 7);
        assert!(console.as_any().downcast_ref::<SpriteConsole>().is_some());
    }

    #[test]
    fn as_any_mut_allows_mutable_downcasting_to_sprite_console() {
        let mut console = SpriteConsole::init(80, 50, 7);
        assert!(
            console
                .as_any_mut()
                .downcast_mut::<SpriteConsole>()
                .is_some()
        );
    }
}
