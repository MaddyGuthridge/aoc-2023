use itertools::Itertools;

fn parse_card(input: &str) -> usize {
    let (winning, actual) = input.split_once(": ").unwrap().1.split_once(" | ").unwrap();

    let winning_numbers = winning
        .split(' ')
        .filter_map(|n| n.trim().parse::<usize>().ok())
        .collect_vec();

    actual
        .split(' ')
        .filter_map(|n| n.trim().parse::<usize>().ok())
        .fold(0, |acc, curr| {
            if winning_numbers.contains(&curr) {
                acc + 1
            } else {
                acc
            }
        })
}

fn calc_card_score(count: usize) -> usize {
    if count == 0 {
        count
    } else {
        (2_usize).pow((count - 1) as u32)
    }
}

#[aoc(day4, part1)]
pub fn part_1(input: &str) -> usize {
    input.lines().map(parse_card).map(calc_card_score).sum()
}

#[aoc(day4, part2)]
pub fn part_2(input: &str) -> usize {
    let parsed_cards = input.lines().map(parse_card).collect_vec();

    let mut upcoming_copies = vec![1; parsed_cards.len()];

    for (i, card_matches) in parsed_cards.into_iter().enumerate() {
        for j in (i + 1)..=usize::min(i + card_matches, upcoming_copies.len()) {
            upcoming_copies[j] += upcoming_copies[i];
        }
    }

    upcoming_copies.iter().sum::<usize>()
}

#[cfg(test)]
mod test {
    use crate::day04::part_2;

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53\n\
            Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19\n\
            Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1\n\
            Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83\n\
            Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36\n\
            Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
            ),
            30
        )
    }
}
