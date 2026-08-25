use bracket_algorithm_traits::prelude::BaseMap;
#[cfg(feature = "threaded")]
use rayon::prelude::*;
#[allow(unused_imports)]
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::convert::TryInto;

/// Representation of a breadth-first flow map.
/// map is a vector of floats, having a size equal to size_x * size_y (one per tile).
/// size_x and size_y are stored for overflow avoidance.
/// max_depth is the maximum number of iterations this search shall support.
pub struct BfsMap {
    pub map: Vec<f32>,
    size_x: usize,
    size_y: usize,
    max_depth: f32,
}

/// Used internally when constructing maps in parallel
#[cfg(feature = "threaded")]
struct ParallelBfs {
    map: Vec<f32>,
    max_depth: f32,
    starts: Vec<usize>,
}

// This is chosen arbitrarily. Whether it's better to
// run threaded or not would depend on map structure,
// map size, number of starts, and probably several
// other parameters. Might want to make this choice
// an explicit part of the API?
#[allow(dead_code)]
const THREADED_REQUIRED_STARTS: usize = 4;

#[derive(PartialEq)]
enum RunThreaded {
    True,
    False,
}

#[allow(dead_code)]
impl BfsMap {
    /// Construct a new BFS map, ready to run. You must specify the map size, and link to an implementation
    /// of a BaseMap trait that can generate exits lists. It then builds the map, giving you a result.
    pub fn new<T>(
        size_x: T,
        size_y: T,
        starts: &[usize],
        map: &dyn BaseMap,
        max_depth: f32,
    ) -> BfsMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        let mut bfs = BfsMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        };
        BfsMap::build(&mut bfs, starts, map);
        bfs
    }

    /// Construct a new BFS map, ready to run. You must specify the map size, and link to an implementation
    /// of a BaseMap trait that can generate exits lists. It then builds the map, giving you a result.
    /// Starts is provided as a set of tuples, two per tile. The first is the tile index, the second the starting
    /// depth (defaults to 0.0 on new).
    pub fn new_weighted<T>(
        size_x: T,
        size_y: T,
        starts: &[(usize, f32)],
        map: &dyn BaseMap,
        max_depth: f32,
    ) -> BfsMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        let mut bfs = BfsMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        };
        BfsMap::build_weighted(&mut bfs, starts, map);
        bfs
    }

    /// Creates an empty BFS map node.
    pub fn new_empty<T>(size_x: T, size_y: T, max_depth: f32) -> BfsMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        BfsMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        }
    }

    /// Clears the BFS map. Uses a parallel for each for performance.
    #[cfg(feature = "threaded")]
    pub fn clear(bfs: &mut BfsMap) {
        bfs.map.par_iter_mut().for_each(|x| *x = f32::MAX);
    }

    #[cfg(not(feature = "threaded"))]
    pub fn clear(bfs: &mut BfsMap) {
        bfs.map.iter_mut().for_each(|x| *x = f32::MAX);
    }

    #[cfg(feature = "threaded")]
    fn build_helper(bfs: &mut BfsMap, starts: &[usize], map: &dyn BaseMap) -> RunThreaded {
        if starts.len() >= THREADED_REQUIRED_STARTS {
            BfsMap::build_parallel(bfs, starts, map);
            return RunThreaded::True;
        }
        RunThreaded::False
    }

    #[cfg(not(feature = "threaded"))]
    fn build_helper(_bfs: &mut BfsMap, _starts: &[usize], _map: &dyn BaseMap) -> RunThreaded {
        RunThreaded::False
    }

    /// Builds the BFS map: iterate from each starting point, to each exit provided by BaseMap's
    /// exits implementation. Each step adds one to the current depth and ignores exit costs.
    /// Automatically branches to a parallel version if you provide more than 4 starting points.
    pub fn build(bfs: &mut BfsMap, starts: &[usize], map: &dyn BaseMap) {
        let threaded = BfsMap::build_helper(bfs, starts, map);
        if threaded == RunThreaded::True {
            return;
        }

        let weighted_starts: Vec<(usize, f32)> = starts.iter().map(|start| (*start, 0.0)).collect();
        BfsMap::build_weighted(bfs, &weighted_starts, map);
    }

    /// Builds the BFS map: iterate from each starting point, to each exit provided by BaseMap's
    /// exits implementation. Each step adds one to the current depth and ignores exit costs.
    pub fn build_weighted(bfs: &mut BfsMap, starts: &[(usize, f32)], map: &dyn BaseMap) {
        let mapsize: usize = bfs.size_x * bfs.size_y;
        let mut open_list: VecDeque<(usize, f32)> = VecDeque::with_capacity(mapsize);

        for (start, depth) in starts.iter().copied() {
            if depth >= bfs.map[start] || depth >= bfs.max_depth {
                continue;
            }
            bfs.map[start] = depth;
            open_list.push_back((start, depth));
        }

        while let Some((tile_idx, depth)) = open_list.pop_front() {
            let exits = map.get_available_exits(tile_idx);
            for (new_idx, _) in exits {
                let new_depth = depth + 1.0;
                let prev_depth = bfs.map[new_idx];
                if new_depth >= prev_depth {
                    continue;
                }
                if new_depth >= bfs.max_depth {
                    continue;
                }
                bfs.map[new_idx] = new_depth;
                open_list.push_back((new_idx, new_depth));
            }
        }
    }

    /// Implementation of Parallel BFS.
    #[cfg(feature = "threaded")]
    fn build_parallel(bfs: &mut BfsMap, starts: &[usize], map: &dyn BaseMap) {
        let mapsize: usize = bfs.size_x * bfs.size_y;
        let mut layers: Vec<ParallelBfs> = Vec::with_capacity(starts.len());
        for start_chunk in starts.chunks(rayon::current_num_threads()) {
            let mut layer = ParallelBfs {
                map: vec![f32::MAX; mapsize],
                max_depth: bfs.max_depth,
                starts: Vec::new(),
            };
            layer.starts.extend(start_chunk.iter().copied());
            layers.push(layer);
        }

        let exits: Vec<SmallVec<[(usize, f32); 10]>> = (0..mapsize)
            .map(|idx| map.get_available_exits(idx))
            .collect();

        // Run each map in parallel.
        layers.par_iter_mut().for_each(|l| {
            let mut open_list: VecDeque<(usize, f32)> = VecDeque::with_capacity(mapsize);

            for start in l.starts.iter().copied() {
                if 0.0 >= l.map[start] || 0.0 >= l.max_depth {
                    continue;
                }
                l.map[start] = 0.0;
                open_list.push_back((start, 0.0));
            }

            while let Some((tile_idx, depth)) = open_list.pop_front() {
                let exits = &exits[tile_idx];
                for (new_idx, _) in exits {
                    let new_idx = *new_idx;
                    let new_depth = depth + 1.0;
                    let prev_depth = l.map[new_idx];
                    if new_depth >= prev_depth {
                        continue;
                    }
                    if new_depth >= l.max_depth {
                        continue;
                    }
                    l.map[new_idx] = new_depth;
                    open_list.push_back((new_idx, new_depth));
                }
            }
        });

        // Recombine down to a single result.
        for l in layers {
            for i in 0..mapsize {
                bfs.map[i] = f32::min(bfs.map[i], l.map[i]);
            }
        }
    }

    /// Helper for traversing maps as path-finding. Provides the index of the lowest available
    /// exit from the specified position index, or None if there isn't one.
    /// You would use this for pathing TOWARDS a starting node.
    #[cfg(feature = "threaded")]
    pub fn find_lowest_exit(bfs: &BfsMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.par_sort_by(|a, b| bfs.map[a.0].partial_cmp(&bfs.map[b.0]).unwrap());

        Some(exits[0].0)
    }

    #[cfg(not(feature = "threaded"))]
    pub fn find_lowest_exit(bfs: &BfsMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.sort_by(|a, b| bfs.map[a.0].partial_cmp(&bfs.map[b.0]).unwrap());

        Some(exits[0].0)
    }

    /// Helper for traversing maps as path-finding. Provides the index of the highest available
    /// exit from the specified position index, or None if there isn't one.
    /// You would use this for pathing AWAY from a starting node, for example if you are running
    /// away.
    #[cfg(feature = "threaded")]
    pub fn find_highest_exit(bfs: &BfsMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.par_sort_by(|a, b| bfs.map[b.0].partial_cmp(&bfs.map[a.0]).unwrap());

        Some(exits[0].0)
    }

    #[cfg(not(feature = "threaded"))]
    pub fn find_highest_exit(bfs: &BfsMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.sort_by(|a, b| bfs.map[b.0].partial_cmp(&bfs.map[a.0]).unwrap());

        Some(exits[0].0)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use bracket_algorithm_traits::prelude::*;

    struct WeightedShortcutMap;

    impl BaseMap for WeightedShortcutMap {
        fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
            match idx {
                0 => smallvec![(1, 10.0), (2, 1.0)],
                2 => smallvec![(1, 1.0)],
                _ => smallvec![],
            }
        }
    }

    #[test]
    fn bfs_counts_edges_not_costs() {
        let map = WeightedShortcutMap;
        let bfs = BfsMap::new(3, 1, &[0], &map, 10.0);

        assert_eq!(bfs.map[0], 0.0);
        assert_eq!(bfs.map[1], 1.0);
        assert_eq!(bfs.map[2], 1.0);
    }
}
