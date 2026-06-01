mod common;
use bracket_color::prelude::*;
use bracket_pathfinding::prelude::*;
use common::*;

fn main() {
    let map = Map::new();

    // Perform the search
    let flow_map = FloydWarshallMap::new(MAP_WIDTH, MAP_HEIGHT, &map, 1024.0);
    let start_idx = map.point2d_to_index(START_POINT);

    // Draw the result
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let idx = y * MAP_WIDTH + x;
            let depth_map_idx = flow_map.idx_helper(start_idx, idx);

            let tile = map.tiles[idx];
            let color = match tile {
                '#' => RGB::named(YELLOW),
                _ => {
                    if flow_map.depth_map[depth_map_idx] < f32::MAX {
                        RGB::from_u8(
                            0,
                            255 - {
                                let n = flow_map.depth_map[depth_map_idx] * 12.0;
                                if n > 255.0 { 255.0 } else { n }
                            } as u8,
                            0,
                        )
                    } else {
                        RGB::named(CHOCOLATE)
                    }
                }
            };
            print_color(color, &tile.to_string());
        }
        print_color(RGB::named(WHITE), "\n");
    }
    flush_console();
}
