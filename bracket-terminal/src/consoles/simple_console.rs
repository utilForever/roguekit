use crate::prelude::{
    CharacterTranslationMode, ColoredTextSpans, Console, FontCharType, TextAlign, Tile,
    string_to_cp437, to_cp437,
};
use bracket_color::prelude::*;
use bracket_geometry::prelude::Rect;
use bracket_rex::prelude::XpLayer;
use std::any::Any;

/// A simple console with background color.
pub struct SimpleConsole {
    pub width: u32,
    pub height: u32,

    pub tiles: Vec<Tile>,
    pub is_dirty: bool,

    // To handle offset tiles for people who want thin walls between tiles
    pub offset_x: f32,
    pub offset_y: f32,

    pub scale: f32,
    pub scale_center: (i32, i32),

    pub extra_clipping: Option<Rect>,
    pub translation: CharacterTranslationMode,
    pub(crate) needs_resize_internal: bool,
}

impl SimpleConsole {
    /// Initializes a console, ready to add to BTerm's console list.
    pub fn init(width: u32, height: u32) -> Box<SimpleConsole> {
        // Console backing init
        let num_tiles: usize = (width * height) as usize;
        let mut tiles: Vec<Tile> = Vec::with_capacity(num_tiles);
        for _ in 0..num_tiles {
            tiles.push(Tile {
                glyph: 0,
                fg: RGBA::from_u8(255, 255, 255, 255),
                bg: RGBA::from_u8(0, 0, 0, 255),
            });
        }

        let new_console = SimpleConsole {
            width,
            height,
            tiles,
            is_dirty: true,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            scale_center: (width as i32 / 2, height as i32 / 2),
            extra_clipping: None,
            translation: CharacterTranslationMode::Codepage437,
            needs_resize_internal: false,
        };

        Box::new(new_console)
    }
}

impl Console for SimpleConsole {
    fn get_char_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize_pixels(&mut self, _width: u32, _height: u32) {
        self.is_dirty = true;
    }

    /// Translate an x/y into an array index.
    fn at(&self, x: i32, y: i32) -> usize {
        (((self.height - 1 - y as u32) * self.width) + x as u32) as usize
    }

    /// Clears the screen.
    fn cls(&mut self) {
        self.is_dirty = true;
        for tile in &mut self.tiles {
            tile.glyph = 32;
            tile.fg = RGBA::from_u8(255, 255, 255, 255);
            tile.bg = RGBA::from_u8(0, 0, 0, 255);
        }
    }

    /// Clears the screen with a background color.
    fn cls_bg(&mut self, background: RGBA) {
        self.is_dirty = true;
        for tile in &mut self.tiles {
            tile.glyph = 32;
            tile.fg = RGBA::from_u8(255, 255, 255, 255);
            tile.bg = background;
        }
    }

    /// Prints a string at x/y.
    fn print(&mut self, mut x: i32, y: i32, output: &str) {
        self.is_dirty = true;
        let bytes = match self.translation {
            CharacterTranslationMode::Codepage437 => string_to_cp437(output),
            CharacterTranslationMode::Unicode => {
                output.chars().map(|c| c as FontCharType).collect()
            }
        };
        for glyph in bytes {
            if let Some(idx) = self.try_at(x, y) {
                self.tiles[idx].glyph = glyph;
            }
            x += 1;
        }
    }

    /// Prints a string at x/y, with foreground and background colors.
    fn print_color(&mut self, mut x: i32, y: i32, fg: RGBA, bg: RGBA, output: &str) {
        self.is_dirty = true;

        let bytes = match self.translation {
            CharacterTranslationMode::Codepage437 => string_to_cp437(output),
            CharacterTranslationMode::Unicode => {
                output.chars().map(|c| c as FontCharType).collect()
            }
        };
        for glyph in bytes {
            if let Some(idx) = self.try_at(x, y) {
                self.tiles[idx].glyph = glyph;
                self.tiles[idx].bg = bg;
                self.tiles[idx].fg = fg;
            }
            x += 1;
        }
    }

    /// Sets a single cell in the console
    fn set(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, glyph: FontCharType) {
        self.is_dirty = true;
        if let Some(idx) = self.try_at(x, y) {
            self.tiles[idx].glyph = glyph;
            self.tiles[idx].fg = fg;
            self.tiles[idx].bg = bg;
        }
    }

    /// Sets a single cell in the console's background
    fn set_bg(&mut self, x: i32, y: i32, bg: RGBA) {
        self.is_dirty = true;
        if let Some(idx) = self.try_at(x, y) {
            self.tiles[idx].bg = bg;
        }
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_box(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_box(self, sx, sy, width, height, fg, bg);
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_hollow_box(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_hollow_box(self, sx, sy, width, height, fg, bg);
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 double line characters
    fn draw_box_double(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_box_double(self, sx, sy, width, height, fg, bg);
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 double line characters
    fn draw_hollow_box_double(
        &mut self,
        sx: i32,
        sy: i32,
        width: i32,
        height: i32,
        fg: RGBA,
        bg: RGBA,
    ) {
        crate::prelude::draw_hollow_box_double(self, sx, sy, width, height, fg, bg);
    }

    /// Fills a rectangle with the specified rendering information
    fn fill_region(&mut self, target: Rect, glyph: FontCharType, fg: RGBA, bg: RGBA) {
        target.for_each(|point| {
            self.set(point.x, point.y, fg, bg, glyph);
        });
    }

    /// Draws a horizontal progress bar
    fn draw_bar_horizontal(
        &mut self,
        sx: i32,
        sy: i32,
        width: i32,
        n: i32,
        max: i32,
        fg: RGBA,
        bg: RGBA,
    ) {
        crate::prelude::draw_bar_horizontal(self, sx, sy, width, n, max, fg, bg);
    }

    /// Draws a vertical progress bar
    fn draw_bar_vertical(
        &mut self,
        sx: i32,
        sy: i32,
        height: i32,
        n: i32,
        max: i32,
        fg: RGBA,
        bg: RGBA,
    ) {
        crate::prelude::draw_bar_vertical(self, sx, sy, height, n, max, fg, bg);
    }

    /// Prints text, centered to the whole console width, at vertical location y.
    fn print_centered(&mut self, y: i32, text: &str) {
        self.is_dirty = true;
        self.print(
            (self.width as i32 / 2) - (text.to_string().len() as i32 / 2),
            y,
            text,
        );
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered(&mut self, y: i32, fg: RGBA, bg: RGBA, text: &str) {
        self.is_dirty = true;
        self.print_color(
            (self.width as i32 / 2) - (text.to_string().len() as i32 / 2),
            y,
            fg,
            bg,
            text,
        );
    }

    /// Prints text, centered to the whole console width, at vertical location y.
    fn print_centered_at(&mut self, x: i32, y: i32, text: &str) {
        self.is_dirty = true;
        self.print(x - (text.to_string().len() as i32 / 2), y, text);
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered_at(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, text: &str) {
        self.is_dirty = true;
        self.print_color(x - (text.to_string().len() as i32 / 2), y, fg, bg, text);
    }

    /// Prints text right-aligned
    fn print_right(&mut self, x: i32, y: i32, text: &str) {
        let len = text.len() as i32;
        let actual_x = x - len;
        self.print(actual_x, y, text);
    }

    /// Prints colored text right-aligned
    fn print_color_right(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, text: &str) {
        let len = text.len() as i32;
        let actual_x = x - len;
        self.print_color(actual_x, y, fg, bg, text);
    }

    /// Print a colorized string with the color encoding defined inline.
    /// For example: printer(1, 1, "#[blue]This blue text contains a #[pink]pink#[] word")
    /// You can get the same effect with a TextBlock, but this can be easier.
    /// Thanks to doryen_rs for the idea.
    fn printer(
        &mut self,
        x: i32,
        y: i32,
        output: &str,
        align: TextAlign,
        background: Option<RGBA>,
    ) {
        let bg = if let Some(bg) = background {
            bg
        } else {
            RGBA::from_u8(0, 0, 0, 255)
        };

        let split_text = ColoredTextSpans::new(output);

        let mut tx = match align {
            TextAlign::Left => x,
            TextAlign::Center => x - (split_text.length as i32 / 2),
            TextAlign::Right => x - split_text.length as i32,
        };
        for span in split_text.spans.iter() {
            let fg = span.0;
            for ch in span.1.chars() {
                self.set(
                    tx,
                    y,
                    fg,
                    bg,
                    match self.translation {
                        CharacterTranslationMode::Codepage437 => to_cp437(ch),
                        CharacterTranslationMode::Unicode => ch as FontCharType,
                    },
                );
                tx += 1;
            }
        }
    }

    /// Saves the layer to an XpFile structure
    fn to_xp_layer(&self) -> XpLayer {
        let mut layer = XpLayer::new(self.width as usize, self.height as usize);

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = layer.get_mut(x as usize, y as usize).unwrap();
                let idx = self.at(x as i32, y as i32);
                cell.ch = u32::from(self.tiles[idx].glyph);
                cell.fg = self.tiles[idx].fg.into();
                cell.bg = self.tiles[idx].bg.into();
            }
        }

        layer
    }

    /// Sets an offset to total console rendering, useful for layers that
    /// draw between tiles. Offsets are specified as a percentage of total
    /// character size; so -0.5 will offset half a character to the left/top.
    fn set_offset(&mut self, x: f32, y: f32) {
        self.is_dirty = true;
        self.offset_x = x * (2.0 / self.width as f32);
        self.offset_y = y * (2.0 / self.height as f32);
    }

    fn set_scale(&mut self, scale: f32, center_x: i32, center_y: i32) {
        self.is_dirty = true;
        self.scale = scale;
        self.scale_center = (center_x, center_y);
    }

    fn get_scale(&self) -> (f32, i32, i32) {
        (self.scale, self.scale_center.0, self.scale_center.1)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    /// Permits the creation of an arbitrary clipping rectangle. It's a really good idea
    /// to make sure that this rectangle is entirely valid.
    fn set_clipping(&mut self, clipping: Option<Rect>) {
        self.extra_clipping = clipping;
    }

    /// Returns the current arbitrary clipping rectangle, None if there isn't one.
    fn get_clipping(&self) -> Option<Rect> {
        self.extra_clipping
    }

    /// Sets ALL tiles foreground alpha (only tiles that exist, in sparse consoles).
    fn set_all_fg_alpha(&mut self, alpha: f32) {
        self.tiles.iter_mut().for_each(|t| t.fg.a = alpha);
    }

    /// Sets ALL tiles background alpha (only tiles that exist, in sparse consoles).
    fn set_all_bg_alpha(&mut self, alpha: f32) {
        self.tiles.iter_mut().for_each(|t| t.bg.a = alpha);
    }

    /// Sets ALL tiles foreground alpha (only tiles that exist, in sparse consoles).
    fn set_all_alpha(&mut self, fg: f32, bg: f32) {
        self.tiles.iter_mut().for_each(|t| {
            t.fg.a = fg;
            t.bg.a = bg;
        });
    }

    /// Sets the character translation mode
    fn set_translation_mode(&mut self, mode: CharacterTranslationMode) {
        self.translation = mode;
    }

    /// Sets the character size of the terminal
    fn set_char_size(&mut self, width: u32, height: u32) {
        // Resize the terminal
        let num_tiles = (width * height) as usize;
        let mut new_tiles: Vec<Tile> = Vec::with_capacity(num_tiles);
        for _ in 0..num_tiles {
            new_tiles.push(Tile {
                glyph: 0,
                fg: RGBA::from_u8(255, 255, 255, 255),
                bg: RGBA::from_u8(0, 0, 0, 255),
            });
        }

        // Copy the old console data
        for y in 0..i32::min(self.height as i32, height as i32) {
            for x in 0..i32::min(self.width as i32, width as i32) {
                let idx = self.at(x, y);
                let new_idx = (((height as i32 - 1 - y) * width as i32) + x) as usize;
                new_tiles[new_idx] = self.tiles[idx];
            }
        }
        self.tiles = new_tiles;

        // Set the size field
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

    #[test]
    fn init_creates_expected_default_state() {
        let console = SimpleConsole::init(80, 50);

        assert_eq!(console.width, 80);
        assert_eq!(console.height, 50);
        assert_eq!(console.tiles.len(), 80 * 50);
        assert!(console.is_dirty);
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
        let console = SimpleConsole::init(10, 4);
        assert_eq!(console.at(x, y), expected);
    }

    #[test]
    fn simple_console_at_mapping_differs_from_test_console_row_major_mapping() {
        let simple = SimpleConsole::init(10, 4);

        assert_eq!(simple.at(0, 0), 30);
        assert_eq!(simple.at(0, 3), 0);
    }

    #[test]
    fn cls_resets_all_tiles_to_space_white_on_black() {
        let mut console = SimpleConsole::init(3, 2);

        console.set(1, 1, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 99);
        console.clear_dirty();
        assert!(!console.is_dirty);

        console.cls();
        assert!(console.is_dirty);
        assert!(console.tiles.iter().all(|tile| {
            tile.glyph == 32 && tile.fg == rgba(255, 255, 255, 255) && tile.bg == rgba(0, 0, 0, 255)
        }));
    }

    #[test]
    fn cls_bg_resets_all_tiles_with_given_background() {
        let mut console = SimpleConsole::init(3, 2);
        let bg = rgba(10, 20, 30, 40);

        console.cls_bg(bg);

        assert!(console.is_dirty);
        assert!(console.tiles.iter().all(|tile| {
            tile.glyph == 32 && tile.fg == rgba(255, 255, 255, 255) && tile.bg == bg
        }));
    }

    #[test]
    fn print_writes_glyphs_but_keeps_existing_colors() {
        let mut console = SimpleConsole::init(5, 2);
        let idx = console.at(1, 0);
        let original_fg = console.tiles[idx].fg;
        let original_bg = console.tiles[idx].bg;

        console.clear_dirty();
        console.print(1, 0, "ABC");

        assert!(console.is_dirty);
        assert_eq!(console.tiles[console.at(1, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(2, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(3, 0)].glyph, 67);
        assert_eq!(console.tiles[idx].fg, original_fg);
        assert_eq!(console.tiles[idx].bg, original_bg);
    }

    #[test]
    fn print_clips_out_of_bounds_characters() {
        let mut console = SimpleConsole::init(3, 1);

        console.print(1, 0, "ABCD");

        assert_eq!(console.tiles[console.at(0, 0)].glyph, 0);
        assert_eq!(console.tiles[console.at(1, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(2, 0)].glyph, 66);
    }

    #[test]
    fn print_color_writes_glyphs_and_colors() {
        let mut console = SimpleConsole::init(5, 2);
        let fg = rgba(1, 2, 3, 4);
        let bg = rgba(5, 6, 7, 8);

        console.print_color(1, 0, fg, bg, "XY");

        let x = console.tiles[console.at(1, 0)];
        let y = console.tiles[console.at(2, 0)];

        assert_eq!(x.glyph, 88);
        assert_eq!(x.fg, fg);
        assert_eq!(x.bg, bg);

        assert_eq!(y.glyph, 89);
        assert_eq!(y.fg, fg);
        assert_eq!(y.bg, bg);
    }

    #[test]
    fn set_writes_single_tile() {
        let mut console = SimpleConsole::init(3, 2);
        let fg = rgba(11, 12, 13, 14);
        let bg = rgba(21, 22, 23, 24);

        console.set(2, 1, fg, bg, 123);

        let tile = console.tiles[console.at(2, 1)];
        assert_eq!(tile.glyph, 123);
        assert_eq!(tile.fg, fg);
        assert_eq!(tile.bg, bg);
    }

    #[test]
    fn set_bg_changes_only_background() {
        let mut console = SimpleConsole::init(3, 2);
        let idx = console.at(1, 1);
        let original_glyph = console.tiles[idx].glyph;
        let original_fg = console.tiles[idx].fg;
        let bg = rgba(9, 8, 7, 6);

        console.set_bg(1, 1, bg);

        assert_eq!(console.tiles[idx].glyph, original_glyph);
        assert_eq!(console.tiles[idx].fg, original_fg);
        assert_eq!(console.tiles[idx].bg, bg);
    }

    #[test]
    fn fill_region_updates_each_tile_in_region() {
        let mut console = SimpleConsole::init(5, 5);
        let fg = rgba(1, 1, 1, 255);
        let bg = rgba(2, 2, 2, 255);

        console.fill_region(Rect::with_size(1, 1, 2, 3), 88, fg, bg);

        for y in 1..4 {
            for x in 1..3 {
                let tile = console.tiles[console.at(x, y)];
                assert_eq!(tile.glyph, 88);
                assert_eq!(tile.fg, fg);
                assert_eq!(tile.bg, bg);
            }
        }

        assert_eq!(console.tiles[console.at(0, 0)].glyph, 0);
    }

    #[test]
    fn print_centered_uses_console_width() {
        let mut console = SimpleConsole::init(10, 2);

        console.print_centered(0, "ABCD");

        assert_eq!(console.tiles[console.at(3, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(4, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(5, 0)].glyph, 67);
        assert_eq!(console.tiles[console.at(6, 0)].glyph, 68);
    }

    #[test]
    fn print_right_ends_before_given_x() {
        let mut console = SimpleConsole::init(10, 2);

        console.print_right(8, 0, "ABC");

        assert_eq!(console.tiles[console.at(5, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(6, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(7, 0)].glyph, 67);
        assert_eq!(console.tiles[console.at(8, 0)].glyph, 0);
    }

    #[test]
    fn set_offset_scales_offsets_by_console_dimensions() {
        let mut console = SimpleConsole::init(10, 20);

        console.set_offset(1.0, -0.5);

        assert_eq!(console.offset_x, 0.2);
        assert_eq!(console.offset_y, -0.05);
        assert!(console.is_dirty);
    }

    #[test]
    fn set_scale_updates_scale_and_center() {
        let mut console = SimpleConsole::init(10, 20);

        console.set_scale(2.5, 3, 4);

        assert_eq!(console.get_scale(), (2.5, 3, 4));
        assert!(console.is_dirty);
    }

    #[test]
    fn clipping_round_trip() {
        let mut console = SimpleConsole::init(10, 20);
        let clipping = Rect::with_size(1, 2, 3, 4);

        assert_eq!(console.get_clipping(), None);

        console.set_clipping(Some(clipping));
        assert_eq!(console.get_clipping(), Some(clipping));
    }

    #[test]
    fn alpha_methods_update_all_tiles() {
        let mut console = SimpleConsole::init(3, 2);

        console.set_all_fg_alpha(0.25);
        assert!(console.tiles.iter().all(|tile| tile.fg.a == 0.25));

        console.set_all_bg_alpha(0.5);
        assert!(console.tiles.iter().all(|tile| tile.bg.a == 0.5));

        console.set_all_alpha(0.75, 1.0);
        assert!(
            console
                .tiles
                .iter()
                .all(|tile| tile.fg.a == 0.75 && tile.bg.a == 1.0)
        );
    }

    #[test]
    fn set_translation_mode_changes_unicode_print_behavior() {
        let mut console = SimpleConsole::init(3, 1);

        console.set_translation_mode(CharacterTranslationMode::Unicode);
        console.print(0, 0, "가");

        assert_eq!(console.tiles[console.at(0, 0)].glyph, '가' as FontCharType);
    }

    #[test]
    fn set_char_size_resizes_buffer_and_preserves_overlapping_content() {
        let mut console = SimpleConsole::init(3, 2);
        let fg = rgba(1, 2, 3, 4);
        let bg = rgba(5, 6, 7, 8);

        console.set(1, 1, fg, bg, 77);
        console.set_char_size(5, 4);

        assert_eq!(console.width, 5);
        assert_eq!(console.height, 4);
        assert_eq!(console.tiles.len(), 20);
        assert!(console.needs_resize_internal);

        let preserved = console.tiles[console.at(1, 1)];
        assert_eq!(preserved.glyph, 77);
        assert_eq!(preserved.fg, fg);
        assert_eq!(preserved.bg, bg);
    }

    #[test]
    fn clear_dirty_resets_dirty_flag() {
        let mut console = SimpleConsole::init(3, 2);
        assert!(console.is_dirty);

        console.clear_dirty();
        assert!(!console.is_dirty);
    }

    #[test]
    fn as_any_allows_downcasting_to_simple_console() {
        let console = SimpleConsole::init(3, 2);
        assert!(console.as_any().downcast_ref::<SimpleConsole>().is_some());
    }

    #[test]
    fn as_any_mut_allows_mutable_downcasting_to_simple_console() {
        let mut console = SimpleConsole::init(3, 2);
        assert!(
            console
                .as_any_mut()
                .downcast_mut::<SimpleConsole>()
                .is_some()
        );
    }
}
