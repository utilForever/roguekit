use bracket_terminal::prelude::*;

bracket_terminal::add_wasm_support!();

struct State {
    player_x: i32,
    player_y: i32,
    glow: bool,
    last_key: Option<VirtualKeyCode>,
}

impl State {
    fn new() -> Self {
        Self {
            player_x: 40,
            player_y: 25,
            glow: false,
            last_key: None,
        }
    }

    fn clamp_player(&mut self) {
        self.player_x = self.player_x.clamp(1, 78);
        self.player_y = self.player_y.clamp(4, 48);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            self.last_key = Some(key);
            match key {
                VirtualKeyCode::Left => self.player_x -= 1,
                VirtualKeyCode::Right => self.player_x += 1,
                VirtualKeyCode::Up => self.player_y -= 1,
                VirtualKeyCode::Down => self.player_y += 1,
                VirtualKeyCode::Space => self.glow = !self.glow,
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
        self.clamp_player();

        let bg = if self.glow {
            RGB::from_u8(8, 22, 40)
        } else {
            RGB::from_u8(0, 0, 0)
        };
        let marker_fg = if self.glow {
            RGB::named(YELLOW)
        } else {
            RGB::named(WHITE)
        };
        let marker_bg = if self.glow {
            RGB::from_u8(20, 60, 100)
        } else {
            RGB::named(BLACK)
        };

        ctx.cls_bg(bg);
        ctx.draw_box(0, 0, 79, 49, RGB::named(CYAN), bg);
        ctx.print_color(2, 1, RGB::named(WHITE), bg, "WebGPU Minimal Example");
        ctx.print_color(
            2,
            2,
            RGB::named(GRAY),
            bg,
            "Arrow keys move, Space toggles glow, Esc exits",
        );

        ctx.print_color(
            2,
            46,
            RGB::named(GREEN),
            bg,
            format!("Mouse tile: {}, {}", ctx.mouse_pos.0, ctx.mouse_pos.1),
        );
        ctx.print_color(
            2,
            47,
            RGB::named(MAGENTA),
            bg,
            format!("Last key: {:?}", self.last_key),
        );
        ctx.print_color(
            2,
            48,
            RGB::named(YELLOW),
            bg,
            format!("FPS: {:.1}  Frame: {:.2} ms", ctx.fps, ctx.frame_time_ms),
        );

        ctx.print_color(self.player_x, self.player_y, marker_fg, marker_bg, "@");
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Bracket Terminal - WebGPU Minimal")
        .build()?;

    main_loop(context, State::new())
}
