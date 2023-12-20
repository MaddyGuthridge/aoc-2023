use std::iter;

use itertools::Itertools;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpringState {
    Safe,
    Damaged,
    Unknown,
}

impl From<char> for SpringState {
    fn from(value: char) -> Self {
        match value {
            '.' => SpringState::Safe,
            '#' => SpringState::Damaged,
            '?' => SpringState::Unknown,
            _ => panic!(),
        }
    }
}

fn load_damages_vec(info: &str) -> Vec<usize> {
    info.split(',').map(|i| i.parse().unwrap()).collect_vec()
}

fn load_spring_states(info: &str) -> Vec<SpringState> {
    info.chars().map_into().collect_vec()
}

/// Returns the remaining slices of the springs and damage info
/// This allows us to skip over checking the starting parts, which we know are
/// correct (since we already checked them earlier in the recursion)
fn get_remaining_springs<'a>(
    springs: &'a mut [SpringState],
    damage_info: &'a [usize],
) -> Option<(&'a mut [SpringState], &'a [usize])> {
    // Current index of damaged spring
    let mut damage_idx = 0;
    let mut start_damaged_idx = 0;
    let mut found_damaged = 0;

    for (springs_idx, spring) in springs.iter().enumerate() {
        let expected_damaged = damage_info.get(damage_idx);
        match spring {
            SpringState::Safe => {
                // If we found some damaged ones
                if found_damaged != 0 {
                    // It must be equal to the expected amount
                    if Some(&found_damaged) != expected_damaged {
                        return None;
                    }
                    damage_idx += 1;
                }
                start_damaged_idx = springs_idx;
                found_damaged = 0;
            }
            SpringState::Damaged => {
                if found_damaged == 0 {
                    start_damaged_idx = springs_idx;
                }
                found_damaged += 1;
                if !expected_damaged.is_some_and(|exp| &found_damaged <= exp) {
                    return None;
                }
            }
            SpringState::Unknown => {
                // No way to tell if it's safe from here on - assume yes
                return Some((&mut springs[start_damaged_idx..], &damage_info[damage_idx..]));
            }
        }
    }

    if let Some(expected) = damage_info.get(damage_idx) {
        if
            // We've found the expected remaining amount of damage
            *expected == found_damaged
            // And there are no remaining expected damage patches
            && damage_idx == damage_info.len() - 1
        {
            Some((&mut [], &[]))
        } else {
            None
        }
    } else if found_damaged == 0 {
        Some((&mut [], &[]))
    } else {
        None
    }
}

/// Returns the index of the next unknown spring state in the slice
fn find_unknown(springs: &[SpringState]) -> usize {
    springs
        .iter()
        .find_position(|s| *s == &SpringState::Unknown)
        .unwrap()
        .0
}

/// Count the number of combinations of spring states that match the given
/// damage info slice
fn count_matching_combos(springs: &mut [SpringState], damage_info: &[usize]) -> usize {
    if let Some((springs, damage_info)) = get_remaining_springs(springs, damage_info) {
        if springs.is_empty() {
            1
        } else {
            let unknown_position = find_unknown(springs);

            // Try all combinations
            let mut count = 0;
            for state in [SpringState::Damaged, SpringState::Safe] {
                springs[unknown_position] = state;
                count += count_matching_combos(springs, damage_info);
            }
            // And reset it back to unknown for the next iteration
            springs[unknown_position] = SpringState::Unknown;
            count
        }
    } else {
        0
    }
}

#[aoc(day12, part1)]
pub fn part_1(input: &str) -> usize {
    input
        .lines()
        .map(|line| line.split_once(' ').unwrap())
        .map(|(springs, damages)| (load_spring_states(springs), load_damages_vec(damages)))
        .map(|(mut springs, damages)| count_matching_combos(&mut springs, &damages))
        .sum()
}

#[aoc(day12, part2)]
pub fn part_2(input: &str) -> usize {
    input
        .lines()
        .map(|line| line.split_once(' ').unwrap())
        .map(|(springs, damages)| (load_spring_states(springs), load_damages_vec(damages)))
        // Repeat the springs and damages 5 times
        .map(|(springs, damages)| {
            (
                // Repeat the springs 5 times
                iter::repeat(springs)
                    .take(5)
                    // Intersperse them with an unknown one
                    .reduce(|mut acc, curr| {
                        acc.push(SpringState::Unknown);
                        acc.extend(curr);
                        acc
                    })
                    .unwrap(),
                // Repeat the damages 5 times
                damages.repeat(5),
            )
        })
        // Do the calculation in parallel, because otherwise it is even slower
        // This still takes about 15 minutes even with this optimisation, but
        // I really can't be bothered to make it any faster
        .enumerate()
        .par_bridge()
        .map(|(i, (mut springs, damages))| (i, count_matching_combos(&mut springs, &damages)))
        .inspect(|(i, e)| {
            println!("[{i}]\t{e}");
        })
        .map(|(_, e)| e)
        .sum()
}

#[cfg(test)]
mod test {
    use super::{part_1, part_2};

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "???.### 1,1,3\n\
                .??..??...?##. 1,1,3\n\
                ?#?#?#?#?#?#?#? 1,3,1,6\n\
                ????.#...#... 4,1,1\n\
                ????.######..#####. 1,6,5\n\
                ?###???????? 3,2,1"
            ),
            21,
        )
    }

    #[test]
    fn test_part_1_simple() {
        assert_eq!(part_1("?###????? 3,2,1"), 1,)
    }

    #[test]
    fn test_part_1_simple_2() {
        assert_eq!(
            part_1(
                "????.######..#####. 1,6,5"
            ),
            4,
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(""), 0)
    }
}
