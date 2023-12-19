use std::{
    collections::HashMap,
    ops::{Add, Mul},
};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<&str> for Direction {
    fn from(value: &str) -> Self {
        match value {
            "U" => UP,
            "D" => DOWN,
            "L" => LEFT,
            "R" => RIGHT,
            _ => panic!("Invalid direction"),
        }
    }
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            '3' => UP,
            '1' => DOWN,
            '2' => LEFT,
            '0' => RIGHT,
            _ => panic!("Invalid direction"),
        }
    }
}

const UP: Direction = Direction::Up;
const DOWN: Direction = Direction::Down;
const LEFT: Direction = Direction::Left;
const RIGHT: Direction = Direction::Right;

struct Displacement {
    direction: Direction,
    amount: i32,
}

type Coord = (i32, i32);
const START_POS: Coord = (0, 0);

impl Add<Direction> for Coord {
    type Output = Coord;

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            UP => (self.0 - 1, self.1),
            DOWN => (self.0 + 1, self.1),
            LEFT => (self.0, self.1 - 1),
            RIGHT => (self.0, self.1 + 1),
        }
    }
}

impl Add<Displacement> for Coord {
    type Output = Coord;

    fn add(self, rhs: Displacement) -> Self::Output {
        match rhs.direction {
            UP => (self.0 - rhs.amount, self.1),
            DOWN => (self.0 + rhs.amount, self.1),
            LEFT => (self.0, self.1 - rhs.amount),
            RIGHT => (self.0, self.1 + rhs.amount),
        }
    }
}

impl Mul<i32> for Direction {
    type Output = Displacement;

    fn mul(self, rhs: i32) -> Self::Output {
        Displacement {
            direction: self,
            amount: rhs,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum TrenchType {
    Downwards,
    Upwards,
}

/// Location of a vertical trench
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct VerticalTrenchLocation {
    /// Horizontal position
    horizontal: i32,
    /// Top location (lowest number)
    top: i32,
    /// Bottom location (highest number)
    bottom: i32,
    /// Type of trench (opening or closing)
    ttype: TrenchType,
}

// Location of a horizontal trench (excluding the row number, since we store that elsewhere)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct HorizontalTrenchLocation(i32, i32);

#[derive(Debug)]
enum TrenchLocation {
    Vertical(VerticalTrenchLocation),
    Horizontal(i32, HorizontalTrenchLocation),
}

fn parse_trench_location_1(
    position: &mut Coord,
    border_positions: &mut (i32, i32),
    input: &str,
) -> TrenchLocation {
    // R 6 (#70c710)
    // ^ ^
    let (dir, amount, _) = input.split(' ').collect_tuple().unwrap();

    let direction = Direction::from(dir);

    let displacement = direction * amount.parse::<i32>().unwrap();

    parse_trench_location_common(position, displacement, direction, border_positions)
}

fn parse_trench_location_2(
    position: &mut Coord,
    border_positions: &mut (i32, i32),
    input: &str,
) -> TrenchLocation {
    //       vvvvvv
    // R 6 (#70c710)
    let info_str = input
        .split(' ')
        .collect_tuple::<(&str, &str, &str)>()
        .unwrap()
        .2
        .replace("(#", "")
        .replace(')', "");

    assert_eq!(info_str.len(), 6);

    //       vvvvv
    // R 6 (#70c710)
    let distance = i32::from_str_radix(&info_str.chars().take(5).collect::<String>(), 16).unwrap();

    //            v
    // R 6 (#70c710)
    let direction = Direction::from(info_str.chars().nth(5).unwrap());

    let displacement = direction * distance;

    parse_trench_location_common(position, displacement, direction, border_positions)
}

fn parse_trench_location_common(
    position: &mut (i32, i32),
    displacement: Displacement,
    direction: Direction,
    border_positions: &mut (i32, i32),
) -> TrenchLocation {
    let result_position = *position + displacement;

    let lower_position = i32::min(result_position.0, position.0);
    let upper_position = i32::max(result_position.0, position.0);

    let leftmost_position = i32::min(result_position.1, position.1);
    let rightmost_position = i32::max(result_position.1, position.1);

    let result = match direction {
        Direction::Up => TrenchLocation::Vertical(VerticalTrenchLocation {
            horizontal: position.1,
            top: lower_position,
            bottom: upper_position,
            ttype: TrenchType::Upwards,
        }),
        Direction::Down => TrenchLocation::Vertical(VerticalTrenchLocation {
            horizontal: position.1,
            top: lower_position,
            bottom: upper_position,
            ttype: TrenchType::Downwards,
        }),
        Direction::Left => TrenchLocation::Horizontal(
            lower_position,
            HorizontalTrenchLocation(leftmost_position, rightmost_position),
        ),
        Direction::Right => TrenchLocation::Horizontal(
            lower_position,
            HorizontalTrenchLocation(leftmost_position, rightmost_position),
        ),
    };

    *position = result_position;

    if position.0 < border_positions.0 {
        border_positions.0 = position.0;
    }
    if position.0 > border_positions.1 {
        border_positions.1 = position.0;
    }

    result
}

fn solve(
    parse_strategy: fn(
        position: &mut Coord,
        border_positions: &mut (i32, i32),
        input: &str,
    ) -> TrenchLocation,
    input: &str,
) -> usize {
    let mut position = START_POS;
    let mut border_positions = (0, 0);

    let trenches = input
        .lines()
        .map(|line| parse_strategy(&mut position, &mut border_positions, line))
        .collect_vec();

    let vertical_trenches = trenches
        .iter()
        .filter_map(|trench| match trench {
            TrenchLocation::Vertical(v) => Some(v),
            TrenchLocation::Horizontal(..) => None,
        })
        .cloned()
        .sorted()
        .collect_vec();

    // Generate horizontal trenches map
    let mut horizontal_trenches: HashMap<i32, Vec<HorizontalTrenchLocation>> = HashMap::default();
    for trench in trenches {
        if let TrenchLocation::Horizontal(row, h) = trench {
            if let Some(v) = horizontal_trenches.get_mut(&row) {
                v.push(h);
            } else {
                horizontal_trenches.insert(row, vec![h]);
            }
        }
    }
    // And sort them
    for v in horizontal_trenches.values_mut() {
        v.sort_unstable();
    }

    let mut num_filled = 0usize;

    for row in border_positions.0..=border_positions.1 {
        let num_filled_row = calc_area_filled_row(
            &vertical_trenches,
            horizontal_trenches.get(&row).unwrap_or(&vec![]),
            row,
        );

        num_filled += num_filled_row;
    }

    num_filled
}

fn calc_area_filled_row(
    vertical_trenches: &[VerticalTrenchLocation],
    horizontal_trenches: &[HorizontalTrenchLocation],
    row: i32,
) -> usize {
    // dbg!(row);
    // Keep track of the direction from which we entered this line
    // so that we can filter out rows we don't care about
    // let prev_type_open: Option<TrenchType> = None;
    // let prev_type_close: Option<TrenchType> = None;
    let intersecting_verticals = vertical_trenches
        .iter()
        .filter(|trench| trench.bottom >= row && trench.top <= row)
        // .filter(filter_consistent_directions(prev_type_open, TrenchType::Opening))
        // // .inspect(|val| {dbg!(val);})
        // .collect_vec()
        // .into_iter()
        // .rev()
        .collect_vec();

    let mut filtered_intersects: Vec<&VerticalTrenchLocation> = vec![];

    let mut open_trench: Option<&VerticalTrenchLocation> = None;

    for vertical in &intersecting_verticals {
        if let Some(opening) = open_trench {
            match opening.ttype {
                TrenchType::Downwards => {
                    match vertical.ttype {
                        TrenchType::Upwards => {
                            // Upwards could potentially close
                            // If it completely goes over this row, it closes
                            if vertical.bottom > row && vertical.top < row {
                                filtered_intersects.push(opening);
                                filtered_intersects.push(vertical);
                                open_trench = None;
                            } else {
                                // Otherwise, if there isn't a horizontal from
                                // here, this is the last one
                                if horizontal_trenches
                                    .binary_search_by(|t| t.0.cmp(&vertical.horizontal))
                                    .is_err()
                                {
                                    filtered_intersects.push(opening);
                                    filtered_intersects.push(vertical);
                                    open_trench = None;
                                }
                            }
                        }
                        TrenchType::Downwards => {
                            // If this is connected to a previous one, then
                            // we're still opening
                            if horizontal_trenches
                                .binary_search_by(|t| t.1.cmp(&vertical.horizontal))
                                .is_ok()
                            {
                                // No action
                            }
                            // Otherwise, if there isn't a horizontal from
                            // here, this is the last one
                            else if horizontal_trenches
                                .binary_search_by(|t| t.0.cmp(&vertical.horizontal))
                                .is_err()
                            {
                                filtered_intersects.push(opening);
                                filtered_intersects.push(vertical);
                                open_trench = None;
                            }
                        }
                    }
                }
                TrenchType::Upwards => {
                    match vertical.ttype {
                        TrenchType::Downwards => {
                            // Downwards could potentially close
                            // If it completely goes over this row, it closes
                            if vertical.bottom > row && vertical.top < row {
                                filtered_intersects.push(opening);
                                filtered_intersects.push(vertical);
                                open_trench = None;
                            } else {
                                // Otherwise, if there isn't a horizontal from
                                // here, this is the last one
                                if horizontal_trenches
                                    .binary_search_by(|t| t.0.cmp(&vertical.horizontal))
                                    .is_err()
                                {
                                    filtered_intersects.push(opening);
                                    filtered_intersects.push(vertical);
                                    open_trench = None;
                                }
                            }
                        }
                        TrenchType::Upwards => {
                            // If this is connected to a previous one
                            if horizontal_trenches
                                .binary_search_by(|t| t.1.cmp(&vertical.horizontal))
                                .is_ok()
                            {
                                // No action
                            }
                            // Otherwise, if there isn't a horizontal from
                            // here, this is the last one
                            else if horizontal_trenches
                                .binary_search_by(|t| t.0.cmp(&vertical.horizontal))
                                .is_err()
                            {
                                filtered_intersects.push(opening);
                                filtered_intersects.push(vertical);
                                open_trench = None;
                            }
                        }
                    }
                }
            }
        } else {
            open_trench = Some(vertical)
        }
    }

    // If there's a remaining open trench, the last one must close it
    if let Some(opening) = open_trench {
        filtered_intersects.push(opening);
        filtered_intersects.push(intersecting_verticals.last().unwrap());
    }

    filtered_intersects
        .into_iter()
        // .filter(filter_consistent_directions(prev_type_close, TrenchType::Closing))
        // .collect_vec()
        // .into_iter()
        // .rev()
        // .inspect(|val| {dbg!(val);})
        .map(|trench| trench.horizontal)
        .tuples::<(i32, i32)>()
        .map(|(start, end)| (end - start + 1) as usize)
        .sum::<usize>()
}

// /// Filter out trenches if they were in the same direction
// fn filter_consistent_directions(
//     mut prev_type: Option<TrenchType>,
//     keep_type: TrenchType,
// ) -> impl FnMut(&&TrenchLocation) -> bool {
//     move |trench| {
//         if let Some(prev) = prev_type {
//             if trench.ttype == prev {
//                 trench.ttype == keep_type
//             } else {
//                 prev_type = Some(trench.ttype);
//                 true
//             }
//         } else {
//             prev_type = Some(trench.ttype);
//             true
//         }
//     }
// }

#[aoc(day18, part1)]
pub fn part_1(input: &str) -> usize {
    solve(parse_trench_location_1, input)
}

#[aoc(day18, part2)]
pub fn part_2(input: &str) -> usize {
    solve(parse_trench_location_2, input)
}

#[cfg(test)]
mod test {
    use crate::day18::HorizontalTrenchLocation;
    use crate::day18::{TrenchType, VerticalTrenchLocation};

    use super::calc_area_filled_row;
    use super::part_1;
    use super::part_2;

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "R 6 (#70c710)\n\
                D 5 (#0dc571)\n\
                L 2 (#5713f0)\n\
                D 2 (#d2c081)\n\
                R 2 (#59c680)\n\
                D 2 (#411b91)\n\
                L 5 (#8ceee2)\n\
                U 2 (#caa173)\n\
                L 1 (#1b58a2)\n\
                U 2 (#caa171)\n\
                R 2 (#7807d2)\n\
                U 3 (#a77fa3)\n\
                L 2 (#015232)\n\
                U 2 (#7a21e3)"
            ),
            62
        )
    }

    #[test]
    fn test_part_1_simple() {
        assert_eq!(
            // #->
            // ^ |
            // <-v
            part_1(
                "R 2 (#70c710)\n\
                D 2 (#0dc571)\n\
                L 2 (#5713f0)\n\
                U 2 (#d2c081)"
            ),
            9
        )
    }

    #[test]
    fn test_part_1_t_shape() {
        assert_eq!(
            // #--v
            // ^<v<
            //  ^<
            part_1(
                "R 3 (#70c710)\n\
                D 1 (#0dc571)\n\
                L 1 (#5713f0)\n\
                D 1 (#d2c081)\n\
                L 1 (#5713f0)\n\
                U 1 (#d2c081)\n\
                L 1 (#d2c081)\n\
                U 1 (#d2c081)"
            ),
            10
        )
    }

    #[test]
    fn test_part_1_l_shape() {
        assert_eq!(
            // #v
            // |>v
            // ^-<
            part_1(
                "R 1 (#70c710)\n\
                D 1 (#0dc571)\n\
                R 1 (#5713f0)\n\
                D 1 (#d2c081)\n\
                L 2 (#5713f0)\n\
                U 2 (#d2c081)"
            ),
            8
        )
    }

    #[test]
    fn test_part_1_j_shape() {
        assert_eq!(
            //  >#
            // >^|
            // ^-<
            part_1(
                "D 2 (#70c710)\n\
                L 2 (#0dc571)\n\
                U 1 (#5713f0)\n\
                R 1 (#d2c081)\n\
                U 1 (#5713f0)\n\
                R 1 (#d2c081)"
            ),
            8
        )
    }

    #[test]
    fn test_part_1_h_shape() {
        assert_eq!(
            // #< v<
            // |^-<|
            // |>-v|
            // >^ >^
            part_1(
                "D 3 (#70c710)\n\
                R 1 (#0dc571)\n\
                U 1 (#5713f0)\n\
                R 2 (#d2c081)\n\
                D 1 (#5713f0)\n\
                R 1 (#5713f0)\n\
                U 3 (#5713f0)\n\
                L 1 (#5713f0)\n\
                D 1 (#5713f0)\n\
                L 2 (#5713f0)\n\
                U 1 (#5713f0)\n\
                L 1 (#5713f0)"
            ),
            18
        )
    }

    #[test]
    fn test_calc_area_extend_end() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: 0,
                        bottom: 2,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 3,
                        top: 1,
                        bottom: 3,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: 0,
                        bottom: 1,
                        ttype: TrenchType::Upwards
                    },
                ],
                &[HorizontalTrenchLocation(3, 4)],
                1,
            ),
            5
        )
    }

    #[test]
    fn test_calc_area_extend_end_inverse() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: 0,
                        bottom: 2,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 3,
                        top: 1,
                        bottom: 3,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: 0,
                        bottom: 1,
                        ttype: TrenchType::Downwards
                    },
                ],
                &[],
                1,
            ),
            5
        )
    }

    #[test]
    fn test_calc_area_extend_start() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: 0,
                        bottom: 2,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 1,
                        top: -1,
                        bottom: 0,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: -2,
                        bottom: 1,
                        ttype: TrenchType::Downwards
                    },
                ],
                &[HorizontalTrenchLocation(0, 1)],
                0,
            ),
            5
        )
    }

    #[test]
    fn test_calc_area_extend_start_inverse() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: 0,
                        bottom: 2,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 1,
                        top: 2,
                        bottom: 3,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: 1,
                        bottom: 3,
                        ttype: TrenchType::Upwards
                    },
                ],
                &[HorizontalTrenchLocation(0, 1)],
                2,
            ),
            5
        )
    }

    #[test]
    fn test_calc_area_wavy() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: 0,
                        bottom: 1,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 1,
                        top: -1,
                        bottom: 0,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 2,
                        top: -1,
                        bottom: 1,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 3,
                        top: -1,
                        bottom: 1,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: -1,
                        bottom: 2,
                        ttype: TrenchType::Downwards
                    },
                ],
                &[],
                0,
            ),
            5
        )
    }

    //     #[test]
    //     fn test_calc_area_wavy_2() {
    //         assert_eq!(
    //             calc_area_filled_row(
    //                 &[
    //                     VerticalTrenchLocation {
    //                         horizontal: 0,
    //                         top: 0,
    //                         bottom: 1,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 3,
    //                         top: -2,
    //                         bottom: 0,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 5,
    //                         top: -2,
    //                         bottom: 0,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 7,
    //                         top: 0,
    //                         bottom: 2,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                 ],
    //                 0,
    //             ),
    //             7
    //         )
    //     }
    //
    //     #[test]
    //     fn test_calc_area_wavy_2_inverse() {
    //         assert_eq!(
    //             calc_area_filled_row(
    //                 &[
    //                     VerticalTrenchLocation {
    //                         horizontal: 0,
    //                         top: 0,
    //                         bottom: 1,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 3,
    //                         top: -2,
    //                         bottom: 0,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 5,
    //                         top: -2,
    //                         bottom: 0,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 7,
    //                         top: 0,
    //                         bottom: 2,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                 ],
    //                 0,
    //             ),
    //             8
    //         )
    //     }
    //
    //     #[test]
    //     fn test_calc_area_wavy_3() {
    //         assert_eq!(
    //             calc_area_filled_row(
    //                 &[
    //                     VerticalTrenchLocation {
    //                         horizontal: 0,
    //                         top: -1,
    //                         bottom: 0,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 2,
    //                         top: -1,
    //                         bottom: 0,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 4,
    //                         top: -1,
    //                         bottom: 0,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 6,
    //                         top: -1,
    //                         bottom: 0,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                 ],
    //                 0,
    //             ),
    //             6
    //         )
    //     }
    //
    #[test]
    fn test_calc_area_wavy_4() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: -3,
                        bottom: 0,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 2,
                        top: -2,
                        bottom: 0,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: -2,
                        bottom: 0,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 6,
                        top: -3,
                        bottom: 0,
                        ttype: TrenchType::Downwards
                    },
                ],
                &[HorizontalTrenchLocation(2, 4)],
                -2,
            ),
            7
        )
    }
    //
    //     #[test]
    //     fn test_calc_area_input_row_6() {
    //         assert_eq!(
    //             calc_area_filled_row(
    //                 &[
    //                     VerticalTrenchLocation {
    //                         horizontal: 0,
    //                         top: 0,
    //                         bottom: 2,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 2,
    //                         top: -3,
    //                         bottom: 0,
    //                         ttype: TrenchType::Upwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 4,
    //                         top: 0,
    //                         bottom: 2,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                     VerticalTrenchLocation {
    //                         horizontal: 6,
    //                         top: -2,
    //                         bottom: 0,
    //                         ttype: TrenchType::Downwards
    //                     },
    //                 ],
    //                 0,
    //             ),
    //             7
    //         )
    //     }
    //
    #[test]
    fn test_calc_area_input_row_8() {
        assert_eq!(
            calc_area_filled_row(
                &[
                    VerticalTrenchLocation {
                        horizontal: 0,
                        top: -2,
                        bottom: 0,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 1,
                        top: 0,
                        bottom: 2,
                        ttype: TrenchType::Upwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 4,
                        top: -2,
                        bottom: 0,
                        ttype: TrenchType::Downwards
                    },
                    VerticalTrenchLocation {
                        horizontal: 6,
                        top: -2,
                        bottom: 0,
                        ttype: TrenchType::Downwards
                    },
                ],
                &[
                    HorizontalTrenchLocation(0, 1),
                    HorizontalTrenchLocation(4, 6),
                ],
                0,
            ),
            7
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "R 6 (#70c710)\n\
                D 5 (#0dc571)\n\
                L 2 (#5713f0)\n\
                D 2 (#d2c081)\n\
                R 2 (#59c680)\n\
                D 2 (#411b91)\n\
                L 5 (#8ceee2)\n\
                U 2 (#caa173)\n\
                L 1 (#1b58a2)\n\
                U 2 (#caa171)\n\
                R 2 (#7807d2)\n\
                U 3 (#a77fa3)\n\
                L 2 (#015232)\n\
                U 2 (#7a21e3)"
            ),
            952408144115
        )
    }
}
