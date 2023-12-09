use itertools::Itertools;

fn read_line(line: &str) -> Vec<i64> {
    line.split(' ').map(|n| n.parse().unwrap()).collect_vec()
}

fn extrapolate_value(values: Vec<i64>) -> i64 {
    let simplified_values = values
        .iter()
        .tuple_windows::<(_, _)>()
        .map(|(a, b)| b - a)
        .collect_vec();

    if simplified_values.iter().sum::<i64>() == 0 {
        values[0]
    } else {
        values[values.len() - 1] + extrapolate_value(simplified_values)
    }
}

fn extrapolate_value_backwards(values: Vec<i64>) -> i64 {
    let simplified_values = values
        .iter()
        .tuple_windows::<(_, _)>()
        .map(|(a, b)| b - a)
        .collect_vec();

    if simplified_values.iter().sum::<i64>() == 0 {
        values[0]
    } else {
        values[0] - extrapolate_value_backwards(simplified_values)
    }
}

#[aoc(day9, part1)]
pub fn part_1(input: &str) -> i64 {
    input.lines().map(read_line).map(extrapolate_value).sum()
}

#[aoc(day9, part2)]
pub fn part_2(input: &str) -> i64 {
    input.lines().map(read_line).map(extrapolate_value_backwards).sum()
}

#[cfg(test)]
mod test {
    use crate::day09::{extrapolate_value, extrapolate_value_backwards};

    #[test]
    fn test_extrapolate() {
        assert_eq!(extrapolate_value(vec![0, 3, 6, 9, 12, 15]), 18)
    }

    #[test]
    fn test_extrapolate_backwards() {
        assert_eq!(extrapolate_value_backwards(vec![0, 3, 6, 9, 12, 15]), -3)
    }
}
