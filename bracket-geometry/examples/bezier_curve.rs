use bracket_geometry::prelude::*;
use crossterm::queue;
use crossterm::style::Print;
use std::io::Write;

const WIDTH: i32 = 40;
const HEIGHT: i32 = 16;

fn main() {
    let curve = Curve::new(vec![
        Point::new(2, 13),
        Point::new(6, 1),
        Point::new(15, 4),
        Point::new(23, 15),
        Point::new(31, 2),
        Point::new(37, 12),
    ]);

    let mut fake_console: Vec<char> = vec!['.'; (WIDTH * HEIGHT) as usize];
    for point in curve.bezier_points(160) {
        if point.x >= 0 && point.x < WIDTH && point.y >= 0 && point.y < HEIGHT {
            let idx = ((point.y * WIDTH) + point.x) as usize;
            fake_console[idx] = '*';
        }
    }

    for control_point in curve.control_points() {
        let idx = ((control_point.y * WIDTH) + control_point.x) as usize;
        fake_console[idx] = 'o';
    }

    for y in 0..HEIGHT {
        let mut line = String::from("");
        let idx = (y * WIDTH) as usize;
        for x in 0..WIDTH as usize {
            line.push(fake_console[idx + x]);
        }
        line.push('\n');
        queue!(std::io::stdout(), Print(&line)).expect("Command fail");
    }
    std::io::stdout().flush().expect("Flush Fail");
}
