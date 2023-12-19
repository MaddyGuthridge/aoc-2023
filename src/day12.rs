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

fn is_state_valid(springs: &[SpringState], damage_info: &[usize]) -> bool {
    let mut damage_iter = damage_info.iter();
    let mut expected_damaged = damage_iter.next();
    let mut found_damaged = 0usize;
    for spring in springs {
        match spring {
            SpringState::Safe => {
                // If we found some damaged ones
                if found_damaged != 0 {
                    // It must be equal to the expected amount
                    if Some(&found_damaged) != expected_damaged {
                        return false;
                    }
                    expected_damaged = damage_iter.next();
                }
                found_damaged = 0;
            }
            SpringState::Damaged => {
                found_damaged += 1;
                if !expected_damaged.is_some_and(|exp| &found_damaged <= exp) {
                    return false;
                }
            }
            SpringState::Unknown => {
                // No way to tell if it's safe from here on - assume yes
                return true;
            }
        }
    }

    if let Some(expected) = expected_damaged {
        // We've found the expected remaining amount of damage
        *expected == found_damaged
        // And there are no remaining expected damage patches
        && damage_iter.count() == 0
    } else {
        found_damaged == 0
    }
}

fn find_unknowns(springs: &[SpringState]) -> Vec<usize> {
    springs
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == &SpringState::Unknown)
        .map(|(i, _)| i)
        .collect_vec()
}

// This is much simpler as a recursive function but is also far slower
// As such, I've written it to use a while loop instead
fn count_matching_combos(springs: &mut [SpringState], damage_info: &[usize]) -> usize {
    let unknowns = find_unknowns(springs);

    let mut num_valid_combinations = 0;
    let mut unknown_index: i32 = 0;

    while unknown_index >= 0 {
        // If there are no more unknowns
        if unknown_index as usize == unknowns.len() {
            // It's complete, add 1 if it's valid
            if is_state_valid(springs, damage_info) {
                num_valid_combinations += 1;
            }
            unknown_index -= 1;
            continue;
        }
        let index = unknowns[unknown_index as usize];
        // If this position is unknown, we haven't checked it yet
        match springs[index] {
            SpringState::Unknown => {
                // First try making it damaged
                springs[index] = SpringState::Damaged;
                // If this gives a valid state, try later combinations
                if is_state_valid(springs, damage_info) {
                    unknown_index += 1;
                } else {
                    // Otherwise, we'll try making it safe on the next iteration
                }
            },
            SpringState::Damaged => {
                // It's currently damaged, so let's now make it safe
                springs[index] = SpringState::Safe;
                // If this gives a valid state, try later combinations
                if is_state_valid(springs, damage_info) {
                    unknown_index += 1;
                } else {
                    // Otherwise, we'll try modifying an earlier one on the
                    // next iteration
                    springs[index] = SpringState::Unknown;
                    unknown_index -= 1;
                }
            },
            SpringState::Safe => {
                // We've tried all combinations from this branch, let's try
                // modifying an earlier one instead
                springs[index] = SpringState::Unknown;
                unknown_index -= 1;
            },
        }
    }

    num_valid_combinations
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
    use super::{is_state_valid, part_1, part_2, SpringState};

    #[test]
    fn test_is_state_valid() {
        assert!(is_state_valid(
            &[
                SpringState::Damaged,
                SpringState::Safe,
                SpringState::Damaged,
            ],
            &[1, 1,],
        ))
    }

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
    fn test_part_2() {
        assert_eq!(part_2(""), 0)
    }
}
