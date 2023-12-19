use std::ops::{Add, Neg};

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

impl Add<(usize, usize)> for Direction {
    type Output = (usize, usize);

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        match self {
            NORTH => (rhs.0 - 1, rhs.1),
            SOUTH => (rhs.0 + 1, rhs.1),
            EAST => (rhs.0, rhs.1 + 1),
            WEST => (rhs.0, rhs.1 - 1),
        }
    }
}

#[derive(Debug, Clone)]
enum Tile {
    Pipe(Direction, Direction),
    Empty,
    Start,
}

impl Tile {
    fn unwrap_pipe(&self) -> (Direction, Direction) {
        if let Tile::Pipe(a, b) = self {
            (*a, *b)
        } else {
            panic!("Not a pipe")
        }
    }
    fn connects_in_dir(&self, dir: Direction) -> bool {
        if let Tile::Pipe(a, b) = self {
            a == &dir || b == &dir
        } else {
            false
        }
    }

    fn get_output_dir(&self, dir: Direction) -> Option<Direction> {
        if let Tile::Pipe(a, b) = self {
            if *a == -dir {
                Some(*b)
            } else if *b == -dir {
                Some(*a)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        match value {
            '|' => Tile::Pipe(NORTH, SOUTH),
            '-' => Tile::Pipe(EAST, WEST),
            'L' => Tile::Pipe(NORTH, EAST),
            'J' => Tile::Pipe(NORTH, WEST),
            '7' => Tile::Pipe(SOUTH, WEST),
            'F' => Tile::Pipe(SOUTH, EAST),
            '.' => Tile::Empty,
            'S' => Tile::Start,
            _ => panic!("Invalid tile"),
        }
    }
}

fn parse_row(line: &str) -> Vec<Tile> {
    line.chars().map(Tile::from).collect_vec()
}

fn get_start(array: &Array2D<Tile>) -> (usize, usize) {
    for ((r, c), tile) in array.enumerate_row_major() {
        if matches!(tile, Tile::Start) {
            return (r, c);
        }
    }

    panic!();
}

/// Traverse until finding the start -- return loop length and other connected
/// direction if there is a complete loop, None if not
fn traverse_loop(
    start: &(usize, usize),
    mut direction: Direction,
    grid: &Array2D<Tile>,
) -> Option<(usize, Direction)> {
    let mut loop_pos = direction + *start;
    let mut num_steps = 1;
    while loop_pos != *start {
        if let Some(cell) = grid.get(loop_pos.0, loop_pos.1) {
            if let Some(output_dir) = cell.get_output_dir(direction) {
                direction = output_dir;
                loop_pos = direction + loop_pos;
                num_steps += 1;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some((num_steps, direction))
}

fn find_loop_length(start: &(usize, usize), grid: &Array2D<Tile>) -> (usize, Tile) {
    // check in each direction
    for direction in [NORTH, EAST, SOUTH, WEST] {
        let result = direction + *start;
        if let Some(cell) = grid.get(result.0, result.1) {
            if cell.connects_in_dir(-direction) {
                if let Some((loop_len, other_direction)) = traverse_loop(start, direction, grid) {
                    return (loop_len, Tile::Pipe(direction, other_direction));
                }
            }
        }
    }

    panic!("Couldn't find a loop direction")
}

#[aoc(day10, part1)]
pub fn part_1(input: &str) -> usize {
    let grid = Array2D::from_rows(&input.lines().map(parse_row).collect_vec()).unwrap();

    let start_position = get_start(&grid);

    find_loop_length(&start_position, &grid).0 / 2
}

/// Create mask where it's true if it's a pipe that is part of the main loop
fn create_pipe_mask(grid: &Array2D<Tile>, start: (usize, usize)) -> Array2D<bool> {
    let mut mask = Array2D::filled_with(false, grid.num_rows(), grid.num_columns());

    mask[start] = true;
    let mut direction = grid[start].unwrap_pipe().0;
    let mut position = direction + start;

    while position != start {
        mask[position] = true;
        direction = grid[position].get_output_dir(direction).unwrap();
        position = direction + position;
    }

    mask
}

#[aoc(day10, part2)]
pub fn part_2(input: &str) -> usize {
    let mut grid = Array2D::from_rows(&input.lines().map(parse_row).collect_vec()).unwrap();

    let start_position = get_start(&grid);

    grid[start_position] = find_loop_length(&start_position, &grid).1;

    let pipe_mask = create_pipe_mask(&grid, start_position);

    let mut num_contained = 0;
    let mut in_loop = false;

    for (position, value) in pipe_mask.enumerate_row_major() {
        if *value {
            // This is part of the main pipe
            let pipe_part = &grid[position];
            // Say if it connects to the north, we've entered/exited the loop
            if pipe_part.connects_in_dir(NORTH) {
                in_loop = !in_loop;
            }
        } else if in_loop {
            num_contained += 1;
        }
    }
    num_contained
}

#[cfg(test)]
mod test {
    use super::part_1;
    // use super::part_2;

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                ".....\n\
                .S-7.\n\
                .|.|.\n\
                .L-J.\n\
                ....."
            ),
            4
        )
    }

    // #[test]
    // fn test_part_2() {
    //     assert_eq!(
    //         part_2(
    //             ".....\n\
    //             .S-7.\n\
    //             .|.|.\n\
    //             .L-J.\n\
    //             ....."
    //         ),
    //         1
    //     )
    // }
}
