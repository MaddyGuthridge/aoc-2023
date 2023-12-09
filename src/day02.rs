const NUM_RED: usize = 12;
const NUM_GREEN: usize = 13;
const NUM_BLUE: usize = 14;

fn was_game_possible(line: &str) -> bool {
    // Get rid of "Day x:"
    let (_, game_data) = line.split_once(": ").unwrap();

    for pull in game_data.split("; ") {
        for group in pull.split(", ") {
            let (count, colour) = group.split_once(' ').unwrap();

            let count = count.parse::<usize>().unwrap();

            if colour == "blue" {
                if count > NUM_BLUE {
                    return false;
                }
            } else if colour == "green" {
                if count > NUM_GREEN {
                    return false;
                }
            } else if colour == "red" {
                if count > NUM_RED {
                    return false;
                }
            } else {
                panic!("Unknown colour {colour} (count {count})");
            }
        }
    }

    true
}

#[aoc(day2, part1)]
pub fn part_1(input: &str) -> usize {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| was_game_possible(line))
        .map(|(i, _)| i + 1)
        .sum()
}

fn calculate_game_power(line: &str) -> usize {
    // Get rid of "Day x:"
    let (_, game_data) = line.split_once(": ").unwrap();

    let mut min_blue = 0;
    let mut min_red = 0;
    let mut min_green = 0;

    for pull in game_data.split("; ") {
        for group in pull.split(", ") {
            let (count, colour) = group.split_once(' ').unwrap();

            let count = count.parse::<usize>().unwrap();

            if colour == "blue" {
                if count > min_blue {
                    min_blue = count;
                }
            } else if colour == "green" {
                if count > min_green {
                    min_green = count;
                }
            } else if colour == "red" {
                if count > min_red {
                    min_red = count;
                }
            } else {
                panic!("Unknown colour {colour} (count {count})");
            }
        }
    }

    min_blue * min_green * min_red
}

#[aoc(day2, part2)]
pub fn part_2(input: &str) -> usize {
    input
        .lines()
        .map(calculate_game_power)
        .sum()
}

#[cfg(test)]
mod test {
    use crate::day02::was_game_possible;

    #[test]
    fn test_basic() {
        assert!(was_game_possible(
            "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green"
        ));
        assert!(!was_game_possible(
            "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red"
        ));
        assert!(!was_game_possible("Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red"))
    }
}
