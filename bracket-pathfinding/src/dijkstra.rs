use bracket_algorithm_traits::prelude::BaseMap;
#[cfg(feature = "threaded")]
use rayon::prelude::*;
#[allow(unused_imports)]
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::convert::TryInto;

/// Representation of a Dijkstra flow map.
/// map is a vector of floats, having a size equal to size_x * size_y (one per tile).
/// size_x and size_y are stored for overflow avoidance.
/// max_depth is the maximum number of iterations this search shall support.
pub struct DijkstraMap {
    pub map: Vec<f32>,
    size_x: usize,
    size_y: usize,
    max_depth: f32,
}

/// Used internally when constructing maps in parallel
#[cfg(feature = "threaded")]
struct ParallelDm {
    map: Vec<f32>,
    max_depth: f32,
    starts: Vec<usize>,
}

#[derive(Copy, Clone, Debug)]
struct DijkstraNode {
    idx: usize,
    cost: f32,
}

impl PartialEq for DijkstraNode {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.cost == other.cost
    }
}

impl Eq for DijkstraNode {}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap()
            .then_with(|| self.idx.cmp(&other.idx))
    }
}

impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
impl DijkstraMap {
    /// Construct a new Dijkstra map, ready to run. You must specify the map size, and link to an implementation
    /// of a BaseMap trait that can generate exits lists. It then builds the map, giving you a result.
    pub fn new<T>(
        size_x: T,
        size_y: T,
        starts: &[usize],
        map: &dyn BaseMap,
        max_depth: f32,
    ) -> DijkstraMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        let mut d = DijkstraMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        };
        DijkstraMap::build(&mut d, starts, map);
        d
    }

    /// Construct a new Dijkstra map, ready to run. You must specify the map size, and link to an implementation
    /// of a BaseMap trait that can generate exits lists. It then builds the map, giving you a result.
    /// Starts is provided as a set of tuples, two per tile. The first is the tile index, the second the starting
    /// weight (defaults to 0.0 on new)
    pub fn new_weighted<T>(
        size_x: T,
        size_y: T,
        starts: &[(usize, f32)],
        map: &dyn BaseMap,
        max_depth: f32,
    ) -> DijkstraMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        let mut d = DijkstraMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        };
        DijkstraMap::build_weighted(&mut d, starts, map);
        d
    }

    /// Creates an empty Dijkstra map node.
    pub fn new_empty<T>(size_x: T, size_y: T, max_depth: f32) -> DijkstraMap
    where
        T: TryInto<usize>,
    {
        let sz_x: usize = size_x.try_into().ok().unwrap();
        let sz_y: usize = size_y.try_into().ok().unwrap();
        let result: Vec<f32> = vec![f32::MAX; sz_x * sz_y];
        DijkstraMap {
            map: result,
            size_x: sz_x,
            size_y: sz_y,
            max_depth,
        }
    }

    /// Clears the Dijkstra map. Uses a parallel for each for performance.
    #[cfg(feature = "threaded")]
    pub fn clear(dm: &mut DijkstraMap) {
        dm.map.par_iter_mut().for_each(|x| *x = f32::MAX);
    }

    #[cfg(not(feature = "threaded"))]
    pub fn clear(dm: &mut DijkstraMap) {
        dm.map.iter_mut().for_each(|x| *x = f32::MAX);
    }

    #[cfg(feature = "threaded")]
    fn build_helper(dm: &mut DijkstraMap, starts: &[usize], map: &dyn BaseMap) -> RunThreaded {
        if starts.len() >= THREADED_REQUIRED_STARTS {
            DijkstraMap::build_parallel(dm, starts, map);
            return RunThreaded::True;
        }
        RunThreaded::False
    }

    #[cfg(not(feature = "threaded"))]
    fn build_helper(_dm: &mut DijkstraMap, _starts: &[usize], _map: &dyn BaseMap) -> RunThreaded {
        RunThreaded::False
    }

    /// Builds the Dijkstra map: iterate from each starting point, to each exit provided by BaseMap's
    /// exits implementation. Each step adds cost to the current depth, and is discarded if the new
    /// depth is further than the current depth.
    /// Automatically branches to a parallel version if you provide more than 4 starting points
    pub fn build(dm: &mut DijkstraMap, starts: &[usize], map: &dyn BaseMap) {
        let threaded = DijkstraMap::build_helper(dm, starts, map);
        if threaded == RunThreaded::True {
            return;
        }

        let weighted_starts: Vec<(usize, f32)> = starts.iter().map(|start| (*start, 0.0)).collect();
        DijkstraMap::build_weighted(dm, &weighted_starts, map);
    }

    /// Builds the Dijkstra map: iterate from each starting point, to each exit provided by BaseMap's
    /// exits implementation. Each step adds cost to the current depth, and is discarded if the new
    /// depth is further than the current depth.
    pub fn build_weighted(dm: &mut DijkstraMap, starts: &[(usize, f32)], map: &dyn BaseMap) {
        let mapsize: usize = dm.size_x * dm.size_y;
        let mut open_list: BinaryHeap<DijkstraNode> = BinaryHeap::with_capacity(mapsize);

        for (start, cost) in starts.iter().copied() {
            if cost >= dm.map[start] || cost >= dm.max_depth {
                continue;
            }
            dm.map[start] = cost;
            open_list.push(DijkstraNode { idx: start, cost });
        }

        while let Some(node) = open_list.pop() {
            if node.cost > dm.map[node.idx] {
                continue;
            }

            let exits = map.get_available_exits(node.idx);
            for (new_idx, add_depth) in exits {
                let new_depth = node.cost + add_depth;
                let prev_depth = dm.map[new_idx];
                if new_depth >= prev_depth {
                    continue;
                }
                if new_depth >= dm.max_depth {
                    continue;
                }
                dm.map[new_idx] = new_depth;
                open_list.push(DijkstraNode {
                    idx: new_idx,
                    cost: new_depth,
                });
            }
        }
    }

    /// Implementation of Parallel Dijkstra.
    #[cfg(feature = "threaded")]
    fn build_parallel(dm: &mut DijkstraMap, starts: &[usize], map: &dyn BaseMap) {
        let mapsize: usize = dm.size_x * dm.size_y;
        let mut layers: Vec<ParallelDm> = Vec::with_capacity(starts.len());
        for start_chunk in starts.chunks(rayon::current_num_threads()) {
            let mut layer = ParallelDm {
                map: vec![f32::MAX; mapsize],
                max_depth: dm.max_depth,
                starts: Vec::new(),
            };
            layer.starts.extend(start_chunk.iter().copied());
            layers.push(layer);
        }

        let exits: Vec<SmallVec<[(usize, f32); 10]>> = (0..mapsize)
            .map(|idx| map.get_available_exits(idx))
            .collect();

        // Run each map in parallel
        layers.par_iter_mut().for_each(|l| {
            let mut open_list: BinaryHeap<DijkstraNode> = BinaryHeap::with_capacity(mapsize);

            for start in l.starts.iter().copied() {
                if 0.0 >= l.map[start] || 0.0 >= l.max_depth {
                    continue;
                }
                l.map[start] = 0.0;
                open_list.push(DijkstraNode {
                    idx: start,
                    cost: 0.0,
                });
            }

            while let Some(node) = open_list.pop() {
                if node.cost > l.map[node.idx] {
                    continue;
                }

                let exits = &exits[node.idx];
                for (new_idx, add_depth) in exits {
                    let new_idx = *new_idx;
                    let new_depth = node.cost + add_depth;
                    let prev_depth = l.map[new_idx];
                    if new_depth >= prev_depth {
                        continue;
                    }
                    if new_depth >= l.max_depth {
                        continue;
                    }
                    l.map[new_idx] = new_depth;
                    open_list.push(DijkstraNode {
                        idx: new_idx,
                        cost: new_depth,
                    });
                }
            }
        });

        // Recombine down to a single result
        for l in layers {
            for i in 0..mapsize {
                dm.map[i] = f32::min(dm.map[i], l.map[i]);
            }
        }
    }

    /// Helper for traversing maps as path-finding. Provides the index of the lowest available
    /// exit from the specified position index, or None if there isn't one.
    /// You would use this for pathing TOWARDS a starting node.
    #[cfg(feature = "threaded")]
    pub fn find_lowest_exit(dm: &DijkstraMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.par_sort_by(|a, b| {
            dm.map[a.0 as usize]
                .partial_cmp(&dm.map[b.0 as usize])
                .unwrap()
        });

        Some(exits[0].0)
    }

    #[cfg(not(feature = "threaded"))]
    pub fn find_lowest_exit(dm: &DijkstraMap, position: usize, map: &dyn BaseMap) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.sort_by(|a, b| dm.map[a.0].partial_cmp(&dm.map[b.0]).unwrap());

        Some(exits[0].0)
    }

    /// Helper for traversing maps as path-finding. Provides the index of the highest available
    /// exit from the specified position index, or None if there isn't one.
    /// You would use this for pathing AWAY from a starting node, for example if you are running
    /// away.
    #[cfg(feature = "threaded")]
    pub fn find_highest_exit(
        dm: &DijkstraMap,
        position: usize,
        map: &dyn BaseMap,
    ) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.par_sort_by(|a, b| {
            dm.map[b.0 as usize]
                .partial_cmp(&dm.map[a.0 as usize])
                .unwrap()
        });

        Some(exits[0].0)
    }

    #[cfg(not(feature = "threaded"))]
    pub fn find_highest_exit(
        dm: &DijkstraMap,
        position: usize,
        map: &dyn BaseMap,
    ) -> Option<usize> {
        let mut exits = map.get_available_exits(position);

        if exits.is_empty() {
            return None;
        }

        exits.sort_by(|a, b| dm.map[b.0].partial_cmp(&dm.map[a.0]).unwrap());

        Some(exits[0].0)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use bracket_algorithm_traits::prelude::*;
    // 1 by 3 stripe of tiles.
    //
    // [0] --1--> [1] --2--> [2]
    // [0] <--1-- [1] <--1-- [2]
    struct MiniMap;
    impl BaseMap for MiniMap {
        fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
            match idx {
                0 => smallvec![(1, 1.)],
                2 => smallvec![(1, 1.)],
                _ => smallvec![(idx - 1, 1.), (idx + 1, 2.)],
            }
        }
    }

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
    fn test_new() {
        let map = MiniMap {};
        let from_zero = DijkstraMap::new(3, 1, &[0], &map, 10.);
        let from_one = DijkstraMap::new(3, 1, &[1], &map, 10.);
        let from_both = DijkstraMap::new(3, 1, &[0, 1], &map, 10.);

        assert_eq!(from_zero.map, vec![0., 1., 3.]);
        assert_eq!(from_one.map, vec![1., 0., 2.]);
        assert_eq!(from_both.map, vec![0., 0., 2.]);
    }

    #[test]
    fn test_new_weighted() {
        let map = MiniMap {};
        let from_zero = DijkstraMap::new_weighted(3, 1, &[(0, 1.)], &map, 10.);
        let from_one = DijkstraMap::new_weighted(3, 1, &[(1, 0.)], &map, 10.);
        let from_both = DijkstraMap::new_weighted(3, 1, &[(0, 1.), (1, 0.)], &map, 10.);

        assert_eq!(from_zero.map, vec![1., 2., 4.]);
        assert_eq!(from_one.map, vec![1., 0., 2.]);
        assert_eq!(from_both.map, vec![1., 0., 2.]);
    }

    #[test]
    fn test_new_empty() {
        let map = DijkstraMap::new_empty(4, 1, 10.);

        assert_eq!(map.map.len(), 4);
        assert!(map.map.iter().all(|depth| *depth == f32::MAX));
    }

    #[test]
    fn test_clear() {
        let mut map = DijkstraMap::new_empty(4, 1, 10.);
        map.map[1] = 1.;
        map.map[3] = 3.;

        DijkstraMap::clear(&mut map);

        assert_eq!(map.map.len(), 4);
        assert!(map.map.iter().all(|depth| *depth == f32::MAX));
    }

    #[test]
    fn test_lowest_exit() {
        let map = MiniMap {};
        let exits_map = DijkstraMap::new(3, 1, &[0], &map, 10.);
        let target = DijkstraMap::find_lowest_exit(&exits_map, 0, &map);
        assert_eq!(target, Some(1));
        let target = DijkstraMap::find_lowest_exit(&exits_map, 1, &map);
        assert_eq!(target, Some(0));
    }

    #[test]
    fn test_highest_exit() {
        let map = MiniMap {};
        let exits_map = DijkstraMap::new(3, 1, &[0], &map, 10.);
        let target = DijkstraMap::find_highest_exit(&exits_map, 0, &map);
        assert_eq!(target, Some(1));
        let target = DijkstraMap::find_highest_exit(&exits_map, 1, &map);
        assert_eq!(target, Some(2));
    }

    #[test]
    fn dijkstra_follows_lowest_total_cost() {
        let map = WeightedShortcutMap;
        let dijkstra = DijkstraMap::new(3, 1, &[0], &map, 10.0);

        assert_eq!(dijkstra.map[0], 0.0);
        assert_eq!(dijkstra.map[1], 2.0);
        assert_eq!(dijkstra.map[2], 1.0);
    }
}
