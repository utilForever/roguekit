//! A virtual console exists to store large amounts of arbitrary text,
//! which can then be "windowed" into actual consoles.

use crate::prelude::{
    BTerm, CharacterTranslationMode, ColoredTextSpans, Console, DrawBatch, FontCharType, TextAlign,
    Tile, string_to_cp437, to_cp437,
};
use bracket_color::prelude::*;
use bracket_geometry::prelude::{Point, Rect};
use bracket_rex::prelude::XpLayer;
use std::any::Any;

pub struct VirtualConsole {
    pub width: u32,
    pub height: u32,

    pub tiles: Vec<Tile>,

    pub extra_clipping: Option<Rect>,
    pub translation: CharacterTranslationMode,
}

impl VirtualConsole {
    /// Creates a new virtual console of arbitrary dimensions.
    pub fn new(dimensions: Point) -> Self {
        let num_tiles: usize = (dimensions.x * dimensions.y) as usize;
        let mut console = VirtualConsole {
            width: dimensions.x as u32,
            height: dimensions.y as u32,
            tiles: Vec::with_capacity(num_tiles),
            extra_clipping: None,
            translation: CharacterTranslationMode::Codepage437,
        };
        for _ in 0..num_tiles {
            console.tiles.push(Tile {
                glyph: 0,
                fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            });
        }
        console
    }

    /// Creates a new virtual console from a blob of text.
    /// Useful if you want to scroll through manuals!
    pub fn from_text(text: &str, width: usize) -> Self {
        let raw_lines = text.split('\n');
        let mut lines: Vec<String> = Vec::new();

        for line in raw_lines {
            let mut newline: String = String::from("");

            for c in line.chars() {
                newline.push(c);

                if newline.len() >= width {
                    lines.push(newline);
                    newline = String::new();
                }
            }

            if !newline.is_empty() {
                lines.push(newline);
            }
        }

        let num_tiles: usize = width * lines.len();
        let mut console = VirtualConsole {
            width: width as u32,
            height: lines.len() as u32,
            tiles: Vec::with_capacity(num_tiles),
            extra_clipping: None,
            translation: CharacterTranslationMode::Codepage437,
        };
        //println!("{}x{}", console.width, console.height);

        for _ in 0..num_tiles {
            console.tiles.push(Tile {
                glyph: 0,
                fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            });
        }

        for (i, line) in lines.iter().enumerate() {
            console.print(0, i as i32, line);
        }

        console
    }

    /// Send a portion of the Virtual Console to a physical console, specifying both source and destination
    /// rectangles and the target console.
    pub fn print_sub_rect(&self, source: Rect, dest: Rect, target: &mut BTerm) {
        target.set_clipping(Some(dest));
        for y in dest.y1..dest.y2 {
            let source_y = y + source.y1 - dest.y1;
            for x in dest.x1..dest.x2 {
                let source_x = x + source.x1 - dest.x1;
                if let Some(idx) = self.try_at(source_x, source_y) {
                    let t = self.tiles[idx];
                    if t.glyph > 0 {
                        target.set(x, y, t.fg, t.bg, t.glyph);
                    }
                }
            }
        }
        target.set_clipping(None);
    }

    /// Send a portion of the Virtual Console to a render batch, specifying both source and destination
    /// rectangles and the target batch.
    pub fn batch_sub_rect(&self, source: Rect, dest: Rect, target: &mut DrawBatch) {
        target.set_clipping(Some(dest));
        for y in dest.y1..dest.y2 {
            let source_y = y + source.y1 - dest.y1;
            for x in dest.x1..dest.x2 {
                let source_x = x + source.x1 - dest.x1;
                if let Some(idx) = self.try_at(source_x, source_y) {
                    let t = self.tiles[idx];
                    if t.glyph > 0 {
                        target.set(Point::new(x, y), ColorPair::new(t.fg, t.bg), t.glyph);
                    }
                }
            }
        }
        target.set_clipping(None);
    }
}

impl Console for VirtualConsole {
    fn get_char_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize_pixels(&mut self, _width: u32, _height: u32) {
        // Ignored
    }

    /// Translate an x/y into an array index.
    fn at(&self, x: i32, y: i32) -> usize {
        (((self.height - 1 - y as u32) * self.width) + x as u32) as usize
    }

    /// Clears the screen.
    fn cls(&mut self) {
        for tile in &mut self.tiles {
            tile.glyph = 32;
            tile.fg = RGBA::from_f32(1.0, 1.0, 1.0, 1.0);
            tile.bg = RGBA::from_f32(0.0, 0.0, 0.0, 1.0);
        }
    }

    /// Clears the screen with a background color.
    fn cls_bg(&mut self, background: RGBA) {
        for tile in &mut self.tiles {
            tile.glyph = 32;
            tile.fg = RGBA::from_f32(1.0, 1.0, 1.0, 1.0);
            tile.bg = background;
        }
    }

    /// Prints a string at x/y.
    fn print(&mut self, mut x: i32, y: i32, output: &str) {
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
        if let Some(idx) = self.try_at(x, y) {
            self.tiles[idx].glyph = glyph;
            self.tiles[idx].fg = fg;
            self.tiles[idx].bg = bg;
        }
    }

    /// Sets a single cell in the console's background
    fn set_bg(&mut self, x: i32, y: i32, bg: RGBA) {
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
        self.print(
            (self.width as i32 / 2) - (text.to_string().len() as i32 / 2),
            y,
            text,
        );
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered(&mut self, y: i32, fg: RGBA, bg: RGBA, text: &str) {
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
        self.print(x - (text.to_string().len() as i32 / 2), y, text);
    }

    /// Prints text in color, centered to the whole console width, at vertical location y.
    fn print_color_centered_at(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, text: &str) {
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
            RGBA::from_f32(0.0, 0.0, 0.0, 1.0)
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
    fn set_offset(&mut self, _x: f32, _y: f32) {
        panic!("Unsupported on virtual consoles.");
    }

    fn set_scale(&mut self, _scale: f32, _center_x: i32, _center_y: i32) {
        panic!("Unsupported on virtual consoles.");
    }

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
    fn set_char_size(&mut self, _width: u32, _height: u32) {
        panic!("Not implemented.");
    }

    // Clears the dirty bit
    fn clear_dirty(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn rgba(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        RGBA::from_f32(r, g, b, a)
    }

    #[test]
    fn new_creates_dense_console_with_expected_dimensions() {
        let console = VirtualConsole::new(Point::new(80, 50));

        assert_eq!(console.get_char_size(), (80, 50));
        assert_eq!(console.tiles.len(), 80 * 50);
        assert_eq!(console.get_clipping(), None);
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
        let console = VirtualConsole::new(Point::new(10, 4));
        assert_eq!(console.at(x, y), expected);
    }

    #[test]
    fn virtual_console_at_mapping_differs_from_test_console_row_major_mapping() {
        let virtual_console = VirtualConsole::new(Point::new(10, 4));

        assert_eq!(virtual_console.at(0, 0), 30);
        assert_eq!(virtual_console.at(0, 3), 0);
    }

    #[test]
    fn cls_resets_all_tiles_to_space_white_on_black() {
        let mut console = VirtualConsole::new(Point::new(3, 2));

        console.set(1, 1, rgba(0.1, 0.2, 0.3, 0.4), rgba(0.5, 0.6, 0.7, 0.8), 99);
        console.cls();

        assert!(console.tiles.iter().all(|tile| {
            tile.glyph == 32
                && tile.fg == rgba(1.0, 1.0, 1.0, 1.0)
                && tile.bg == rgba(0.0, 0.0, 0.0, 1.0)
        }));
    }

    #[test]
    fn cls_bg_resets_all_tiles_with_given_background() {
        let mut console = VirtualConsole::new(Point::new(3, 2));
        let bg = rgba(0.1, 0.2, 0.3, 0.4);

        console.cls_bg(bg);

        assert!(console.tiles.iter().all(|tile| {
            tile.glyph == 32 && tile.fg == rgba(1.0, 1.0, 1.0, 1.0) && tile.bg == bg
        }));
    }

    #[test]
    fn print_writes_glyphs_but_keeps_existing_colors() {
        let mut console = VirtualConsole::new(Point::new(5, 2));
        let idx = console.at(1, 0);
        let original_fg = console.tiles[idx].fg;
        let original_bg = console.tiles[idx].bg;

        console.print(1, 0, "ABC");

        assert_eq!(console.tiles[console.at(1, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(2, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(3, 0)].glyph, 67);
        assert_eq!(console.tiles[idx].fg, original_fg);
        assert_eq!(console.tiles[idx].bg, original_bg);
    }

    #[test]
    fn print_clips_out_of_bounds_characters() {
        let mut console = VirtualConsole::new(Point::new(3, 1));

        console.print(1, 0, "ABCD");

        assert_eq!(console.tiles[console.at(0, 0)].glyph, 0);
        assert_eq!(console.tiles[console.at(1, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(2, 0)].glyph, 66);
    }

    #[test]
    fn print_color_writes_glyphs_and_colors() {
        let mut console = VirtualConsole::new(Point::new(5, 2));
        let fg = rgba(0.1, 0.2, 0.3, 0.4);
        let bg = rgba(0.5, 0.6, 0.7, 0.8);

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
        let mut console = VirtualConsole::new(Point::new(3, 2));
        let fg = rgba(0.1, 0.2, 0.3, 0.4);
        let bg = rgba(0.5, 0.6, 0.7, 0.8);

        console.set(2, 1, fg, bg, 123);

        let tile = console.tiles[console.at(2, 1)];
        assert_eq!(tile.glyph, 123);
        assert_eq!(tile.fg, fg);
        assert_eq!(tile.bg, bg);
    }

    #[test]
    fn set_ignores_out_of_bounds_coordinates() {
        let mut console = VirtualConsole::new(Point::new(3, 1));

        console.set(3, 0, rgba(0.1, 0.2, 0.3, 0.4), rgba(0.5, 0.6, 0.7, 0.8), 65);
        console.set(
            -1,
            0,
            rgba(0.1, 0.2, 0.3, 0.4),
            rgba(0.5, 0.6, 0.7, 0.8),
            66,
        );
        console.set(0, 1, rgba(0.1, 0.2, 0.3, 0.4), rgba(0.5, 0.6, 0.7, 0.8), 67);

        assert!(console.tiles.iter().all(|tile| tile.glyph == 0));
    }

    #[test]
    fn set_bg_changes_only_background() {
        let mut console = VirtualConsole::new(Point::new(3, 2));
        let idx = console.at(1, 1);
        let original_glyph = console.tiles[idx].glyph;
        let original_fg = console.tiles[idx].fg;
        let bg = rgba(0.9, 0.8, 0.7, 0.6);

        console.set_bg(1, 1, bg);

        assert_eq!(console.tiles[idx].glyph, original_glyph);
        assert_eq!(console.tiles[idx].fg, original_fg);
        assert_eq!(console.tiles[idx].bg, bg);
    }

    #[test]
    fn fill_region_updates_each_tile_in_region() {
        let mut console = VirtualConsole::new(Point::new(5, 5));
        let fg = rgba(0.1, 0.1, 0.1, 1.0);
        let bg = rgba(0.2, 0.2, 0.2, 1.0);

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
        let mut console = VirtualConsole::new(Point::new(10, 2));

        console.print_centered(0, "ABCD");

        assert_eq!(console.tiles[console.at(3, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(4, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(5, 0)].glyph, 67);
        assert_eq!(console.tiles[console.at(6, 0)].glyph, 68);
    }

    #[test]
    fn print_right_ends_before_given_x() {
        let mut console = VirtualConsole::new(Point::new(10, 2));

        console.print_right(8, 0, "ABC");

        assert_eq!(console.tiles[console.at(5, 0)].glyph, 65);
        assert_eq!(console.tiles[console.at(6, 0)].glyph, 66);
        assert_eq!(console.tiles[console.at(7, 0)].glyph, 67);
        assert_eq!(console.tiles[console.at(8, 0)].glyph, 0);
    }

    #[test]
    fn clipping_round_trip() {
        let mut console = VirtualConsole::new(Point::new(10, 20));
        let clipping = Rect::with_size(1, 2, 3, 4);

        assert_eq!(console.get_clipping(), None);

        console.set_clipping(Some(clipping));
        assert_eq!(console.get_clipping(), Some(clipping));
    }

    #[test]
    fn clipping_limits_set_and_print() {
        let mut console = VirtualConsole::new(Point::new(10, 5));

        console.set_clipping(Some(Rect::with_size(2, 1, 3, 2)));
        console.print(0, 1, "ABCDE");
        console.set(4, 2, rgba(0.1, 0.2, 0.3, 0.4), rgba(0.5, 0.6, 0.7, 0.8), 90);
        console.set(5, 2, rgba(0.1, 0.2, 0.3, 0.4), rgba(0.5, 0.6, 0.7, 0.8), 91);

        assert_eq!(console.tiles[console.at(0, 1)].glyph, 0);
        assert_eq!(console.tiles[console.at(1, 1)].glyph, 0);
        assert_eq!(console.tiles[console.at(2, 1)].glyph, 67);
        assert_eq!(console.tiles[console.at(3, 1)].glyph, 68);
        assert_eq!(console.tiles[console.at(4, 1)].glyph, 69);
        assert_eq!(console.tiles[console.at(4, 2)].glyph, 90);
        assert_eq!(console.tiles[console.at(5, 2)].glyph, 0);
    }

    #[test]
    fn alpha_methods_update_all_tiles() {
        let mut console = VirtualConsole::new(Point::new(3, 2));

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
        let mut console = VirtualConsole::new(Point::new(3, 1));

        console.set_translation_mode(CharacterTranslationMode::Unicode);
        console.print(0, 0, "가");

        assert_eq!(console.tiles[console.at(0, 0)].glyph, '가' as FontCharType);
    }

    #[test]
    fn from_text_wraps_lines_and_prints_content() {
        let console = VirtualConsole::from_text("abcdef\ngh", 3);

        assert_eq!(console.width, 3);
        assert_eq!(console.height, 3);
        assert_eq!(console.tiles[console.at(0, 0)].glyph, 97);
        assert_eq!(console.tiles[console.at(1, 0)].glyph, 98);
        assert_eq!(console.tiles[console.at(2, 0)].glyph, 99);
        assert_eq!(console.tiles[console.at(0, 1)].glyph, 100);
        assert_eq!(console.tiles[console.at(1, 1)].glyph, 101);
        assert_eq!(console.tiles[console.at(2, 1)].glyph, 102);
        assert_eq!(console.tiles[console.at(0, 2)].glyph, 103);
        assert_eq!(console.tiles[console.at(1, 2)].glyph, 104);
    }

    #[test]
    fn resize_pixels_is_ignored() {
        let mut console = VirtualConsole::new(Point::new(10, 20));

        console.resize_pixels(100, 200);

        assert_eq!(console.get_char_size(), (10, 20));
        assert_eq!(console.tiles.len(), 200);
    }

    #[test]
    fn get_scale_returns_fixed_default() {
        let console = VirtualConsole::new(Point::new(10, 20));
        assert_eq!(console.get_scale(), (1.0, 0, 0));
    }

    #[test]
    #[should_panic(expected = "Unsupported on virtual consoles.")]
    fn set_offset_panics() {
        let mut console = VirtualConsole::new(Point::new(10, 20));
        console.set_offset(1.0, 2.0);
    }

    #[test]
    #[should_panic(expected = "Unsupported on virtual consoles.")]
    fn set_scale_panics() {
        let mut console = VirtualConsole::new(Point::new(10, 20));
        console.set_scale(2.0, 3, 4);
    }

    #[test]
    #[should_panic(expected = "Not implemented.")]
    fn set_char_size_panics() {
        let mut console = VirtualConsole::new(Point::new(10, 20));
        console.set_char_size(30, 40);
    }

    #[test]
    fn clear_dirty_is_no_op() {
        let mut console = VirtualConsole::new(Point::new(10, 20));

        console.clear_dirty();
        assert_eq!(console.get_char_size(), (10, 20));
    }

    #[test]
    fn as_any_allows_downcasting_to_virtual_console() {
        let console = VirtualConsole::new(Point::new(10, 20));
        assert!(console.as_any().downcast_ref::<VirtualConsole>().is_some());
    }

    #[test]
    fn as_any_mut_allows_mutable_downcasting_to_virtual_console() {
        let mut console = VirtualConsole::new(Point::new(10, 20));

        assert!(
            console
                .as_any_mut()
                .downcast_mut::<VirtualConsole>()
                .is_some()
        );
    }
}
