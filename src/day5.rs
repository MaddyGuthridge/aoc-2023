use std::str::Lines;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Range {
    start: i64,
    length: i64,
}

#[derive(Debug)]
struct RangeMap {
    input: i64,
    output: i64,
    length: i64,
}

impl RangeMap {
    fn get(&self, input: i64) -> Option<i64> {
        if self.input <= input && input < self.input + self.length {
            Some(input - self.input + self.output)
        } else {
            None
        }
    }

    // Return (mapped range, any remaining range)
    fn get_range(&self, input: &Range) -> (Option<Range>, Option<Range>) {
        // Start in range
        if self.input <= input.start && input.start < self.input + self.length {
            // Fully in range
            if input.start + input.length <= self.input + self.length {
                (
                    Some(Range {
                        start: input.start - self.input + self.output,
                        length: input.length,
                    }),
                    None,
                )
            } else {
                // Doesn't fit fully in range
                let new_length =
                    input.length - ((input.start + input.length) - (self.input + self.length));

                (
                    // Mapped range
                    Some(Range {
                        start: input.start - self.input + self.output,
                        length: new_length,
                    }),
                    // Remaining range
                    Some(Range {
                        start: input.start + new_length,
                        length: input.length - new_length,
                    }),
                )
            }
        } else {
            (None, Some(input.clone()))
        }
    }
}

impl From<&str> for RangeMap {
    fn from(value: &str) -> Self {
        let (output, input, length) = value.split(' ').collect_tuple().unwrap();

        Self {
            input: input.parse().unwrap(),
            output: output.parse().unwrap(),
            length: length.parse().unwrap(),
        }
    }
}

fn parse_seed_list(line: &str) -> Vec<i64> {
    line.split_once(": ")
        .unwrap()
        .1
        .split(' ')
        .map(|n| n.parse().unwrap())
        .collect_vec()
}

fn parse_seed_list_part_2(line: &str) -> Vec<Range> {
    parse_seed_list(line)
        .chunks(2)
        .map(|c| {
            let start = c[0];
            let length = c[1];
            Range { start, length }
        })
        .collect_vec()
}

fn read_mapping(lines: &mut Lines<'_>) -> Option<Vec<RangeMap>> {
    if !lines.next()?.contains(" map:") {
        return None;
    }

    let mut mappings = vec![];

    for line in lines {
        if line.is_empty() {
            break;
        }
        mappings.push(RangeMap::from(line));
    }

    Some(mappings)
}

#[aoc(day5, part1)]
pub fn part_1(input: &str) -> i64 {
    let mut lines = input.lines();

    let mut seeds = parse_seed_list(lines.next().unwrap());

    // Skip empty line
    lines.next();

    let mut mappings_vec = vec![];

    while let Some(map) = read_mapping(&mut lines) {
        mappings_vec.push(map);
    }

    for mapping in mappings_vec {
        let mut new_values = vec![];

        for seed in seeds {
            let mut new_val = None;
            for range in &mapping {
                new_val = range.get(seed);
                if new_val.is_some() {
                    break;
                }
            }
            new_values.push(new_val.unwrap_or(seed));
        }

        seeds = new_values;
    }

    seeds.iter().min().unwrap().to_owned()
}

fn transpose_range(range: Range, mappings: &Vec<RangeMap>) -> Vec<Range> {
    for map in mappings {
        match map.get_range(&range) {
            (None, None) => panic!("BRUH"),
            (None, Some(_)) => continue,
            (Some(output), None) => return vec![output],
            (Some(output), Some(remainder)) => {
                let mut v = vec![output];

                // Handle the remaining range
                v.extend(transpose_range(remainder, mappings));

                return v;
            }
        }
    }

    vec![range]
}

#[aoc(day5, part2)]
pub fn part_2(input: &str) -> i64 {
    let mut lines = input.lines();

    let mut seeds = parse_seed_list_part_2(lines.next().unwrap());

    // Skip empty line
    lines.next();

    let mut mappings_vec = vec![];

    while let Some(map) = read_mapping(&mut lines) {
        mappings_vec.push(map);
    }

    for mapping in mappings_vec {
        let mut new_values = vec![];

        for seed_range in seeds {
            new_values.extend(transpose_range(seed_range, &mapping));
        }

        seeds = new_values;
    }

    seeds
        .iter()
        .flat_map(|r| (r.start)..(r.start + r.length))
        .min()
        .unwrap()
        .to_owned()
}

#[cfg(test)]
mod test {
    // use crate::day5::part_2;

    use crate::day5::{part_1, part_2};

    use super::{Range, RangeMap};

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "seeds: 79 14 55 13\n\
                \n\
                seed-to-soil map:\n\
                50 98 2\n\
                52 50 48\n\
                \n\
                soil-to-fertilizer map:\n\
                0 15 37\n\
                37 52 2\n\
                39 0 15\n\
                \n\
                fertilizer-to-water map:\n\
                49 53 8\n\
                0 11 42\n\
                42 0 7\n\
                57 7 4\n\
                \n\
                water-to-light map:\n\
                88 18 7\n\
                18 25 70\n\
                \n\
                light-to-temperature map:\n\
                45 77 23\n\
                81 45 19\n\
                68 64 13\n\
                \n\
                temperature-to-humidity map:\n\
                0 69 1\n\
                1 0 69\n\
                \n\
                humidity-to-location map:\n\
                60 56 37\n\
                56 93 4"
            ),
            35,
        );
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "seeds: 79 14 55 13\n\
                \n\
                seed-to-soil map:\n\
                50 98 2\n\
                52 50 48\n\
                \n\
                soil-to-fertilizer map:\n\
                0 15 37\n\
                37 52 2\n\
                39 0 15\n\
                \n\
                fertilizer-to-water map:\n\
                49 53 8\n\
                0 11 42\n\
                42 0 7\n\
                57 7 4\n\
                \n\
                water-to-light map:\n\
                88 18 7\n\
                18 25 70\n\
                \n\
                light-to-temperature map:\n\
                45 77 23\n\
                81 45 19\n\
                68 64 13\n\
                \n\
                temperature-to-humidity map:\n\
                0 69 1\n\
                1 0 69\n\
                \n\
                humidity-to-location map:\n\
                60 56 37\n\
                56 93 4"
            ),
            46,
        );
    }

    #[test]
    fn test_map_range_full() {
        let r = Range { start: 0, length: 5 };

        let map = RangeMap { input: 0, output: 1, length: 5 };

        assert_eq!(
            map.get_range(&r),
            (
                Some(Range { start: 1, length: 5 }),
                None
            )
        )
    }

    #[test]
    fn test_map_range_none() {
        let r = Range { start: 0, length: 5 };

        let map = RangeMap { input: 5, output: 1, length: 5 };

        assert_eq!(
            map.get_range(&r),
            (
                None,
                Some(r)
            )
        )
    }

    #[test]
    fn test_map_range_partial() {
        let r = Range { start: 3, length: 5 };

        let map = RangeMap { input: 0, output: 1, length: 5 };

        assert_eq!(
            map.get_range(&r),
            (
                Some(Range { start: 4, length: 2 }),
                Some(Range { start: 5, length: 3 })
            )
        )
    }
}
