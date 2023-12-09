use itertools::Itertools;

fn read_row(line: &str) -> Vec<i64> {
    line.split_once(": ")
        .unwrap()
        .1
        .trim()
        .split_ascii_whitespace()
        .map(|n| n.parse().unwrap())
        .collect()
}

fn num_winning_combos(time: i64, distance: i64) -> i64 {
    let mut num_wins = 0;
    for wait_time in 1..(time - 1) {
        let our_dist = wait_time * (time - wait_time);
        if our_dist > distance {
            num_wins += 1;
        }
    }
    num_wins
}

#[aoc(day6, part1)]
pub fn part_1(input: &str) -> i64 {
    let (times, distances) = input.lines().collect_tuple().unwrap();

    let times = read_row(times);
    let distances = read_row(distances);

    times
        .into_iter()
        .zip(distances)
        .map(|(t, d)| num_winning_combos(t, d))
        .product()
}

#[aoc(day6, part2)]
pub fn part_2(input: &str) -> i64 {
    let (times, distances) = input.lines().collect_tuple().unwrap();

    let time = times.split_once(": ").unwrap().1.replace(' ', "").parse().unwrap();
    let distance = distances.split_once(": ").unwrap().1.replace(' ', "").parse().unwrap();

    num_winning_combos(time, distance)
}
