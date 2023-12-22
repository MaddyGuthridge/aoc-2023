//! Since all vertical and horizontal paths from the start are open in the
//! input, and the Elf can only move horizontally and vertically, we can safely
//! assume that if they can make it from one corner to the other, they can fill
//! all tiles in the area - as such, we'll just fill starting from each
//! corner and the centre of each edge, and that will be enough for us to
//! calculate the fills.
//!
//! To do this, we imagine the explored world's chunks as follows:
//!
//!      -3-2-1 0 1 2 3
//!     +--------------
//!   -3|     L A C
//!   -2|   L K * B C
//!   -1| L K * * * B C
//!    0| J * * S * * D
//!    1| H I * * * E F
//!    2|   H I * E F
//!    3|     H G F
//!
//! In this diagram, S represents the starting point, * represents
//! fully-explored chunks. Other letters are partially filled. For example,
//!
//! * A's fill starts from the bottom middle
//! * B and C's fills start from the bottom left
//! * D's fill starts from the middle left
//!
//! And so on...
//!
//! As such, we only need to calculate fills for the tiles with letters A-L,
//! greatly reducing the required compute time
use std::{
    collections::VecDeque,
    ops::{Add, Index, IndexMut, Neg},
};

use array2d::Array2D;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

const NORTH: Direction = Direction::North;
const SOUTH: Direction = Direction::South;
const EAST: Direction = Direction::East;
const WEST: Direction = Direction::West;

impl Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        match self {
            NORTH => SOUTH,
            SOUTH => NORTH,
            EAST => WEST,
            WEST => EAST,
        }
    }
}

impl Add<Direction> for (i32, i32) {
    type Output = (i32, i32);

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            NORTH => (self.0 - 1, self.1),
            SOUTH => (self.0 + 1, self.1),
            EAST => (self.0, self.1 + 1),
            WEST => (self.0, self.1 - 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Start,
    Rock,
    /// Garden that hasn't been visited yet
    Unvisited,
    /// Garden that was visited at a given number of steps
    Visited(usize),
}

impl Tile {
    /// Return whether a tile can be visited
    fn is_visitable(&self) -> bool {
        [Tile::Start, Tile::Unvisited].contains(self)
    }
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        match value {
            'S' => Tile::Start,
            '#' => Tile::Rock,
            '.' => Tile::Unvisited,
            c => panic!("Invalid tile {c}"),
        }
    }
}

#[derive(Debug, Clone)]
struct Chunk(Array2D<Tile>);

impl Chunk {
    /// Parse the given chunk from the input string
    fn parse(input: &str) -> Chunk {
        Chunk(
            Array2D::from_rows(
                &input
                    .lines()
                    .map(|line| line.chars().map(Tile::from).collect_vec())
                    .collect_vec(),
            )
            .unwrap(),
        )
    }

    /// Length of each side of the chunk
    #[inline]
    fn dimensions(&self) -> i32 {
        self.0.num_rows() as i32
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Tile> + Clone {
        self.0.elements_row_major_iter()
    }

    fn get(&self, (r, c): (i32, i32)) -> Option<&Tile> {
        self.0.get(r as usize, c as usize)
    }

    /// Find the starting position in the chunk
    fn start_index(&self) -> (i32, i32) {
        // Start is guaranteed to be in the centre of the world
        let start = (self.dimensions() / 2, self.dimensions() / 2);
        assert!(matches!(self[start], Tile::Start));
        start
    }

    /// Use BFS algorithm to fill the given world
    fn fill(mut self, start: (i32, i32)) -> Self {
        let mut q = VecDeque::default();
        q.push_back((start, 0));
        while let Some((location, depth)) = q.pop_front() {
            // If we've already visited it, ignore it
            if !self[location].is_visitable() {
                continue;
            }
            // Otherwise, visit it now
            self[location] = Tile::Visited(depth);
            // Check all directions
            for direction in [NORTH, EAST, SOUTH, WEST] {
                let result = location + direction;
                if let Some(result_tile) = self.get(result) {
                    if result_tile.is_visitable() {
                        q.push_back((result, depth + 1))
                    }
                }
            }
        }

        self
    }

    /// For the given max depth, return the number of tiles that are visitable in
    /// the chunk, accounting for the evenness of the given depth
    fn num_tiles_visitable_at_depth(&self, max_depth: i32) -> usize {
        if max_depth < 0 {
            return 0;
        }
        let max_depth = max_depth as usize;
        let evenness = max_depth % 2;
        self.iter()
            .filter(|tile| {
                if let Tile::Visited(count) = **tile {
                    // If we've visited, check whether it matches the evenness
                    count % 2 == evenness && count <= max_depth
                } else {
                    false
                }
            })
            .count()
    }
}

impl Index<(i32, i32)> for Chunk {
    type Output = Tile;

    fn index(&self, (r, c): (i32, i32)) -> &Self::Output {
        &self.0[(r as usize, c as usize)]
    }
}

impl IndexMut<(i32, i32)> for Chunk {
    fn index_mut(&mut self, (r, c): (i32, i32)) -> &mut Self::Output {
        &mut self.0[(r as usize, c as usize)]
    }
}

/// Representation of the infinite world
#[derive(Debug, Clone)]
struct InfiniteWorld {
    /// Dimensions of each chunk
    chunk_size: i32,

    // Number of items in a completely filled chunk
    num_even: usize,
    num_odd: usize,

    // Centre approaches
    world_north: Chunk,
    world_east: Chunk,
    world_south: Chunk,
    world_west: Chunk,

    // Corner approaches
    world_north_east: Chunk,
    world_north_west: Chunk,
    world_south_east: Chunk,
    world_south_west: Chunk,
}

impl InfiniteWorld {
    fn new(chunk: Chunk) -> Self {
        let mid = chunk.dimensions() / 2;
        let end = chunk.dimensions() - 1;
        let filled = chunk.clone().fill(chunk.start_index());

        InfiniteWorld {
            chunk_size: chunk.dimensions(),

            // Just hard-code the depths to be more than the width
            num_even: filled.num_tiles_visitable_at_depth(1000),
            num_odd: filled.num_tiles_visitable_at_depth(1001),

            // Fill the other chunks for each approach direction
            world_north: chunk.clone().fill((0, mid)),
            world_east: chunk.clone().fill((mid, end)),
            world_south: chunk.clone().fill((end, mid)),
            world_west: chunk.clone().fill((mid, 0)),
            world_north_east: chunk.clone().fill((0, end)),
            world_north_west: chunk.clone().fill((0, 0)),
            world_south_east: chunk.clone().fill((end, end)),
            world_south_west: chunk.clone().fill((end, 0)),
        }
    }

    /// Return a reference to the chunk filled from the given approach direction
    fn get_filled_chunk(
        &self,
        approach: Direction,
        secondary_approach: Option<Direction>,
    ) -> &Chunk {
        match approach {
            Direction::North => match secondary_approach {
                Some(s) => match s {
                    Direction::East => &self.world_north_east,
                    Direction::West => &self.world_north_west,
                    _ => panic!(),
                },
                None => &self.world_north,
            },
            Direction::South => match secondary_approach {
                Some(s) => match s {
                    Direction::East => &self.world_south_east,
                    Direction::West => &self.world_south_west,
                    _ => panic!(),
                },
                None => &self.world_south,
            },
            Direction::East => {
                assert!(secondary_approach.is_none());
                &self.world_east
            }
            Direction::West => {
                assert!(secondary_approach.is_none());
                &self.world_west
            }
        }
    }

    /// Return the number of chunks that can be fully covered in a single
    /// direction, excluding the starting chunk
    fn num_chunks_covered_in_a_single_direction(&self, mut num_steps: usize) -> usize {
        if (num_steps as i32) < self.chunk_size {
            return 0;
        }
        // one half for the distance to the edge of the chunk,
        // one half for the distance to the corner of the chunk
        // Go down to nearest even, because the map is an odd width, meaning
        // it'll be slightly less than the full width
        num_steps -= self.chunk_size as usize - 1;

        // Now return the number of times we can fully cross a chunk
        num_steps / self.chunk_size as usize
    }

    /// Return the number of steps remaining after walking to the closest
    /// corner/edge of a chunk. Includes the step into the first cell of the
    /// chunk.
    ///
    /// Examples
    ///
    ///     +--+
    ///     |  |<------S
    ///     +--+
    ///
    ///
    ///        +-------S
    ///        |
    ///        v
    ///     +--+
    ///     |  |
    ///     +--+
    fn steps_remaining_at_chunk(&self, num_steps: usize, (row, col): (i32, i32)) -> i32 {
        // Number of full chunks covered (excluding starting chunk)
        let chunks_covered =
            row.abs() + col.abs() - (if row == 0 { 0 } else { 1 } + if col == 0 { 0 } else { 1 });

        // Steps required to leave the starting chunk
        let steps_from_starting_chunk = (if row == 0 { 0 } else { self.chunk_size / 2 + 1 })
            + (if col == 0 { 0 } else { self.chunk_size / 2 + 1 });

        num_steps as i32 - chunks_covered * self.chunk_size - steps_from_starting_chunk
    }
}

#[aoc(day21, part1)]
pub fn part_1(input: &str) -> usize {
    let chunk = Chunk::parse(input);
    let start = chunk.start_index();
    chunk.fill(start).num_tiles_visitable_at_depth(64)
}

fn num_positions_after_steps(input: &str, num_steps: usize) -> usize {
    let world = InfiniteWorld::new(Chunk::parse(input));

    let explored_width = world.num_chunks_covered_in_a_single_direction(num_steps);

    if explored_width == 0 {
        // Didn't explore past the starting chunk - just use the part 1
        // approach
        let chunk = Chunk::parse(input);
        let start = chunk.start_index();
        return chunk
            .fill(start)
            .num_tiles_visitable_at_depth(num_steps as i32);
    }

    // Create a diamond of fully explored chunks
    // Depending on the dimensions of each chunk, the odd and even squares are
    // alternated, as per this diagram
    //
    //   O
    //  OEO
    // OEOEO
    //  OEO
    //   O
    //
    // Note that although I'm calling these "odd" and "even", they may be the
    // other way around depending on the number of steps - as long as they
    // match up with `world.num_even` and `world.num_odd` it'll add up fine
    let num_odd_chunks = (0..(explored_width)).sum::<usize>() * 2 + explored_width;
    let num_even_chunks = (1..=(explored_width)).sum::<usize>() * 2 + explored_width + 1;

    // The sum of all chunks so far
    let mut sum = world.num_even * num_even_chunks + world.num_odd * num_odd_chunks;

    // Number of steps when entering the corner chunks at the end of the
    // diamond
    // This is the same at all corners
    let remaining_steps_at_points =
        world.steps_remaining_at_chunk(num_steps, (0, explored_width as i32 + 1));

    for dir in [NORTH, EAST, SOUTH, WEST] {
        sum += world
            .get_filled_chunk(dir, None)
            .num_tiles_visitable_at_depth(remaining_steps_at_points);
    }

    // Number of partially-visited chunks on each diagonal
    let num_close_diagonals = explored_width;
    let num_far_diagonals = explored_width + 1;

    let remaining_steps_at_close_diagonal =
        world.steps_remaining_at_chunk(num_steps, (1, explored_width as i32));
    let remaining_steps_at_far_diagonal =
        world.steps_remaining_at_chunk(num_steps, (1, explored_width as i32 + 1));

    for diagonal in [(NORTH, EAST), (NORTH, WEST), (SOUTH, EAST), (SOUTH, WEST)] {
        // Close diagonal
        sum += world
            .get_filled_chunk(diagonal.0, Some(diagonal.1))
            .num_tiles_visitable_at_depth(remaining_steps_at_close_diagonal)
            * num_close_diagonals;

        // Far diagonal
        sum += world
            .get_filled_chunk(diagonal.0, Some(diagonal.1))
            .num_tiles_visitable_at_depth(remaining_steps_at_far_diagonal)
            * num_far_diagonals;
    }

    sum
}

#[aoc(day21, part2)]
pub fn part_2(input: &str) -> usize {
    num_positions_after_steps(input, 26501365)
}

#[cfg(test)]
mod test {
    use super::num_positions_after_steps;

    fn num_positions_with_simple_input(num_steps: usize) -> usize {
        num_positions_after_steps(
            "...\n\
             .S.\n\
             ...",
            num_steps,
        )
    }

    fn num_positions_with_complex_input(num_steps: usize) -> usize {
        num_positions_after_steps(
            ".....\n\
             .#.#.\n\
             ..S..\n\
             .#.#.\n\
             .....",
            num_steps,
        )
    }

    #[test]
    fn test_part_2_simple() {
        assert_eq!(num_positions_with_complex_input(2), 5);
    }

    #[test]
    fn test_part_2_wrap_next_cell() {
        assert_eq!(num_positions_with_simple_input(7), 64);
    }
}
