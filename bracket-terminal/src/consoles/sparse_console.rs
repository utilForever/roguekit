use crate::prelude::{
    CharacterTranslationMode, ColoredTextSpans, Console, FontCharType, TextAlign, string_to_cp437,
    to_cp437,
};
use bracket_color::prelude::RGBA;
use bracket_geometry::prelude::Rect;
use bracket_rex::prelude::XpLayer;
use std::any::Any;

/// Internal storage structure for sparse tiles.
#[derive(Clone, Copy, PartialEq)]
pub struct SparseTile {
    pub idx: usize,
    pub glyph: FontCharType,
    pub fg: RGBA,
    pub bg: RGBA,
}

/// A sparse console. Rather than storing every cell on the screen, it stores just cells that have
/// data.
pub struct SparseConsole {
    pub width: u32,
    pub height: u32,

    pub tiles: Vec<SparseTile>,
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

impl SparseConsole {
    /// Initializes the console.
    pub fn init(width: u32, height: u32) -> Box<SparseConsole> {
        // Console backing init
        let new_console = SparseConsole {
            width,
            height,
            tiles: Vec::with_capacity((width * height) as usize),
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

impl Console for SparseConsole {
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
        self.tiles.clear();
    }

    /// Clear the screen. Since we don't HAVE a background, it doesn't use it.
    fn cls_bg(&mut self, _background: RGBA) {
        self.is_dirty = true;
        self.tiles.clear();
    }

    /// Prints a string to an x/y position.
    fn print(&mut self, x: i32, y: i32, output: &str) {
        self.is_dirty = true;

        let bounds = self.get_char_size();
        let bytes = match self.translation {
            CharacterTranslationMode::Codepage437 => string_to_cp437(output),
            CharacterTranslationMode::Unicode => {
                output.chars().map(|c| c as FontCharType).collect()
            }
        };

        self.tiles.extend(
            bytes
                .into_iter()
                .enumerate()
                .filter(|(i, _)| (*i as i32 + x) < bounds.0 as i32)
                .map(|(i, glyph)| {
                    let idx =
                        (((bounds.1 - 1 - y as u32) * bounds.0) + (x + i as i32) as u32) as usize;
                    SparseTile {
                        idx,
                        glyph,
                        fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                        bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
                    }
                }),
        );
    }

    /// Prints a string to an x/y position, with foreground and background colors.
    fn print_color(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, output: &str) {
        self.is_dirty = true;

        let bounds = self.get_char_size();
        let bytes = match self.translation {
            CharacterTranslationMode::Codepage437 => string_to_cp437(output),
            CharacterTranslationMode::Unicode => {
                output.chars().map(|c| c as FontCharType).collect()
            }
        };
        self.tiles.extend(
            bytes
                .into_iter()
                .enumerate()
                .filter(|(i, _)| (*i as i32 + x) < bounds.0 as i32)
                .map(|(i, glyph)| {
                    let idx =
                        (((bounds.1 - 1 - y as u32) * bounds.0) + (x + i as i32) as u32) as usize;
                    SparseTile { idx, glyph, fg, bg }
                }),
        );
    }

    /// Sets a single cell in the console
    fn set(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, glyph: FontCharType) {
        self.is_dirty = true;
        if let Some(idx) = self.try_at(x, y) {
            self.tiles.push(SparseTile { idx, glyph, fg, bg });
        }
    }

    /// Sets a single cell in the console's background
    fn set_bg(&mut self, x: i32, y: i32, bg: RGBA) {
        if let Some(idx) = self.try_at(x, y) {
            self.is_dirty = true;
            let mut found_tile = false;
            self.tiles
                .iter_mut()
                .filter(|t| t.idx == idx)
                .for_each(|t| {
                    t.bg = bg;
                    found_tile = true;
                });
            if !found_tile {
                self.tiles.push(SparseTile {
                    idx,
                    glyph: match self.translation {
                        CharacterTranslationMode::Codepage437 => to_cp437(' '),
                        CharacterTranslationMode::Unicode => ' ' as FontCharType,
                    },
                    fg: RGBA::from_u8(0, 0, 0, 255),
                    bg,
                });
            }
        }
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_box(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_box(self, sx, sy, width, height, fg, bg);
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 double line characters
    fn draw_box_double(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_box_double(self, sx, sy, width, height, fg, bg);
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_hollow_box(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        crate::prelude::draw_hollow_box(self, sx, sy, width, height, fg, bg);
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

        // Clear all to transparent
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = layer.get_mut(x as usize, y as usize).unwrap();
                cell.bg = bracket_rex::prelude::XpColor::TRANSPARENT;
            }
        }

        for c in &self.tiles {
            let x = c.idx % self.width as usize;
            let y = c.idx / self.width as usize;
            let cell = layer.get_mut(x, y).unwrap();
            cell.ch = u32::from(c.glyph);
            cell.fg = c.fg.into();
            cell.bg = c.bg.into();
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
    fn init_creates_empty_sparse_console() {
        let console = SparseConsole::init(80, 50);

        assert_eq!(console.get_char_size(), (80, 50));
        assert!(console.tiles.is_empty());
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
        let console = SparseConsole::init(10, 4);
        assert_eq!(console.at(x, y), expected);
    }

    #[test]
    fn sparse_console_at_mapping_differs_from_test_console_row_major_mapping() {
        let sparse = SparseConsole::init(10, 4);

        assert_eq!(sparse.at(0, 0), 30);
        assert_eq!(sparse.at(0, 3), 0);
    }

    #[test]
    fn print_appends_sparse_tiles_with_default_colors() {
        let mut console = SparseConsole::init(10, 4);

        console.clear_dirty();
        console.print(2, 1, "AB");

        assert!(console.is_dirty);
        assert_eq!(console.tiles.len(), 2);
        assert_eq!(console.tiles[0].idx, console.at(2, 1));
        assert_eq!(console.tiles[0].glyph, 65);
        assert_eq!(console.tiles[0].fg, RGBA::from_f32(1.0, 1.0, 1.0, 1.0));
        assert_eq!(console.tiles[0].bg, RGBA::from_f32(0.0, 0.0, 0.0, 1.0));
        assert_eq!(console.tiles[1].idx, console.at(3, 1));
        assert_eq!(console.tiles[1].glyph, 66);
    }

    #[test]
    fn print_clips_at_right_edge() {
        let mut console = SparseConsole::init(3, 1);

        console.print(1, 0, "ABCD");

        assert_eq!(console.tiles.len(), 2);
        assert_eq!(console.tiles[0].idx, console.at(1, 0));
        assert_eq!(console.tiles[0].glyph, 65);
        assert_eq!(console.tiles[1].idx, console.at(2, 0));
        assert_eq!(console.tiles[1].glyph, 66);
    }

    #[test]
    fn print_color_appends_sparse_tiles_with_given_colors() {
        let mut console = SparseConsole::init(10, 4);
        let fg = rgba(1, 2, 3, 4);
        let bg = rgba(5, 6, 7, 8);

        console.print_color(1, 2, fg, bg, "XY");

        assert_eq!(console.tiles.len(), 2);
        assert_eq!(console.tiles[0].idx, console.at(1, 2));
        assert_eq!(console.tiles[0].glyph, 88);
        assert_eq!(console.tiles[0].fg, fg);
        assert_eq!(console.tiles[0].bg, bg);
        assert_eq!(console.tiles[1].idx, console.at(2, 2));
        assert_eq!(console.tiles[1].glyph, 89);
        assert_eq!(console.tiles[1].fg, fg);
        assert_eq!(console.tiles[1].bg, bg);
    }

    #[test]
    fn set_appends_sparse_tile_when_in_bounds() {
        let mut console = SparseConsole::init(10, 4);
        let fg = rgba(11, 12, 13, 14);
        let bg = rgba(21, 22, 23, 24);

        console.set(2, 1, fg, bg, 123);

        assert_eq!(console.tiles.len(), 1);
        assert_eq!(console.tiles[0].idx, console.at(2, 1));
        assert_eq!(console.tiles[0].glyph, 123);
        assert_eq!(console.tiles[0].fg, fg);
        assert_eq!(console.tiles[0].bg, bg);
    }

    #[test]
    fn set_ignores_out_of_bounds_coordinates() {
        let mut console = SparseConsole::init(3, 1);

        console.set(3, 0, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 65);
        console.set(-1, 0, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 66);
        console.set(0, 1, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 67);

        assert!(console.tiles.is_empty());
    }

    #[test]
    fn set_bg_updates_existing_tile_backgrounds_with_same_idx() {
        let mut console = SparseConsole::init(10, 4);
        let old_bg = rgba(1, 1, 1, 255);
        let new_bg = rgba(9, 8, 7, 6);

        console.set(2, 1, rgba(1, 2, 3, 4), old_bg, 65);
        console.set(2, 1, rgba(5, 6, 7, 8), old_bg, 66);
        console.set_bg(2, 1, new_bg);

        assert_eq!(console.tiles.len(), 2);
        assert!(
            console
                .tiles
                .iter()
                .all(|tile| tile.idx == console.at(2, 1) && tile.bg == new_bg)
        );
    }

    #[test]
    fn set_bg_creates_space_tile_when_no_existing_tile_matches() {
        let mut console = SparseConsole::init(10, 4);
        let bg = rgba(9, 8, 7, 6);

        console.set_bg(2, 1, bg);

        assert_eq!(console.tiles.len(), 1);
        assert_eq!(console.tiles[0].idx, console.at(2, 1));
        assert_eq!(console.tiles[0].glyph, to_cp437(' '));
        assert_eq!(console.tiles[0].fg, rgba(0, 0, 0, 255));
        assert_eq!(console.tiles[0].bg, bg);
    }

    #[test]
    fn set_bg_ignores_out_of_bounds_coordinates() {
        let mut console = SparseConsole::init(3, 1);

        console.set_bg(3, 0, rgba(9, 8, 7, 6));
        console.set_bg(-1, 0, rgba(9, 8, 7, 6));
        console.set_bg(0, 1, rgba(9, 8, 7, 6));

        assert!(console.tiles.is_empty());
    }

    #[test]
    fn cls_clears_sparse_tiles_and_marks_dirty() {
        let mut console = SparseConsole::init(10, 4);

        console.print(0, 0, "ABC");
        console.clear_dirty();
        assert!(!console.is_dirty);

        console.cls();
        assert!(console.is_dirty);
        assert!(console.tiles.is_empty());
    }

    #[test]
    fn cls_bg_clears_sparse_tiles_and_ignores_background() {
        let mut console = SparseConsole::init(10, 4);

        console.print(0, 0, "ABC");
        console.clear_dirty();
        assert!(!console.is_dirty);

        console.cls_bg(rgba(1, 2, 3, 4));
        assert!(console.is_dirty);
        assert!(console.tiles.is_empty());
    }

    #[test]
    fn fill_region_adds_one_tile_per_in_bounds_cell() {
        let mut console = SparseConsole::init(5, 5);
        let fg = rgba(1, 1, 1, 255);
        let bg = rgba(2, 2, 2, 255);

        console.fill_region(Rect::with_size(1, 1, 2, 3), 88, fg, bg);

        assert_eq!(console.tiles.len(), 6);
        assert!(
            console
                .tiles
                .iter()
                .all(|tile| tile.glyph == 88 && tile.fg == fg && tile.bg == bg)
        );
    }

    #[test]
    fn print_centered_uses_console_width() {
        let mut console = SparseConsole::init(10, 2);

        console.print_centered(0, "ABCD");

        assert_eq!(console.tiles.len(), 4);
        assert_eq!(console.tiles[0].idx, console.at(3, 0));
        assert_eq!(console.tiles[1].idx, console.at(4, 0));
        assert_eq!(console.tiles[2].idx, console.at(5, 0));
        assert_eq!(console.tiles[3].idx, console.at(6, 0));
    }

    #[test]
    fn print_right_ends_before_given_x() {
        let mut console = SparseConsole::init(10, 2);

        console.print_right(8, 0, "ABC");

        assert_eq!(console.tiles.len(), 3);
        assert_eq!(console.tiles[0].idx, console.at(5, 0));
        assert_eq!(console.tiles[1].idx, console.at(6, 0));
        assert_eq!(console.tiles[2].idx, console.at(7, 0));
    }

    #[test]
    fn printer_supports_inline_colored_text() {
        let mut console = SparseConsole::init(20, 2);

        console.printer(0, 0, "#[red]A#[]B", TextAlign::Left, None);

        assert_eq!(console.tiles.len(), 2);
        assert_eq!(console.tiles[0].glyph, to_cp437('A'));
        assert_eq!(console.tiles[1].glyph, to_cp437('B'));
    }

    #[test]
    fn set_offset_scales_offsets_by_console_dimensions() {
        let mut console = SparseConsole::init(10, 20);

        console.set_offset(1.0, -0.5);

        assert_eq!(console.offset_x, 0.2);
        assert_eq!(console.offset_y, -0.05);
        assert!(console.is_dirty);
    }

    #[test]
    fn set_scale_updates_scale_and_center() {
        let mut console = SparseConsole::init(10, 20);

        console.set_scale(2.5, 3, 4);

        assert_eq!(console.get_scale(), (2.5, 3, 4));
        assert!(console.is_dirty);
    }

    #[test]
    fn clipping_round_trip() {
        let mut console = SparseConsole::init(10, 20);
        let clipping = Rect::with_size(1, 2, 3, 4);

        assert_eq!(console.get_clipping(), None);

        console.set_clipping(Some(clipping));
        assert_eq!(console.get_clipping(), Some(clipping));
    }

    #[test]
    fn clipping_limits_set() {
        let mut console = SparseConsole::init(10, 5);

        console.set_clipping(Some(Rect::with_size(2, 1, 3, 2)));
        console.set(1, 1, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 65);
        console.set(2, 1, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 66);
        console.set(4, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 67);
        console.set(5, 2, rgba(1, 2, 3, 4), rgba(5, 6, 7, 8), 68);

        assert_eq!(console.tiles.len(), 2);
        assert_eq!(console.tiles[0].glyph, 66);
        assert_eq!(console.tiles[1].glyph, 67);
    }

    #[test]
    fn alpha_methods_update_existing_tiles_only() {
        let mut console = SparseConsole::init(10, 4);

        console.print(0, 0, "AB");

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
        let mut console = SparseConsole::init(3, 1);

        console.set_translation_mode(CharacterTranslationMode::Unicode);
        console.print(0, 0, "가");

        assert_eq!(console.tiles[0].glyph, '가' as FontCharType);
    }

    #[test]
    fn set_char_size_updates_dimensions_without_rebuilding_tiles() {
        let mut console = SparseConsole::init(3, 2);

        console.print(0, 0, "A");
        console.set_char_size(5, 4);

        assert_eq!(console.get_char_size(), (5, 4));
        assert_eq!(console.tiles.len(), 1);
        assert!(console.needs_resize_internal);
    }

    #[test]
    fn resize_pixels_marks_dirty_without_changing_dimensions() {
        let mut console = SparseConsole::init(80, 50);

        console.clear_dirty();
        console.resize_pixels(1024, 768);

        assert!(console.is_dirty);
        assert_eq!(console.get_char_size(), (80, 50));
    }

    #[test]
    fn clear_dirty_resets_dirty_flag() {
        let mut console = SparseConsole::init(80, 50);
        assert!(console.is_dirty);

        console.clear_dirty();
        assert!(!console.is_dirty);
    }

    #[test]
    fn as_any_allows_downcasting_to_sparse_console() {
        let console = SparseConsole::init(80, 50);
        assert!(console.as_any().downcast_ref::<SparseConsole>().is_some());
    }

    #[test]
    fn as_any_mut_allows_mutable_downcasting_to_sparse_console() {
        let mut console = SparseConsole::init(80, 50);
        assert!(
            console
                .as_any_mut()
                .downcast_mut::<SparseConsole>()
                .is_some()
        );
    }
}
