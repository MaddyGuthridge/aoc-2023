use std::cmp::Ordering;

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Card {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
    J,
    Q,
    K,
    A,
}

impl From<char> for Card {
    fn from(value: char) -> Self {
        match value {
            'A' => Card::A,
            'K' => Card::K,
            'Q' => Card::Q,
            'J' => Card::J,
            'T' => Card::T,
            '9' => Card::N9,
            '8' => Card::N8,
            '7' => Card::N7,
            '6' => Card::N6,
            '5' => Card::N5,
            '4' => Card::N4,
            '3' => Card::N3,
            '2' => Card::N2,
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug)]
struct Hand([Card; 5]);

// impl Hand {
//     fn get_hand_type(&self) -> HandType {
//         let mut card_counts: HashMap<Card, u8> = HashMap::default();
//
//         for card in self.0 {
//             card_counts.insert(card, card_counts.get(&card).unwrap_or(&0) + 1);
//         }
//
//         let mut num_pairs = 0;
//         let mut num_triples = 0;
//
//         for count in card_counts.values() {
//             match count {
//                 2 => {
//                     num_pairs += 1;
//                 }
//                 3 => {
//                     num_triples += 1;
//                 }
//                 4 => {
//                     return HandType::FourOfAKind;
//                 }
//                 5 => {
//                     return HandType::FiveOfAKind;
//                 }
//                 _ => {}
//             }
//         }
//
//         if num_pairs == 2 {
//             HandType::TwoPair
//         } else if num_pairs == 1 {
//             if num_triples == 1 {
//                 HandType::FullHouse
//             } else {
//                 HandType::OnePair
//             }
//         } else if num_triples == 1 {
//             HandType::ThreeOfAKind
//         } else {
//             HandType::HighCard
//         }
//     }
// }

impl From<&Hand> for HandType {
    fn from(value: &Hand) -> Self {
        match value.0.iter()
            .sorted()
            .group_by(|v| *v)
            .into_iter()
            .map(|(_, v)| v.count())
            .sorted()
            .collect_vec()[..] {
                [5] => HandType::FiveOfAKind,
                [1, 4] => HandType::FourOfAKind,
                [2, 3] => HandType::FullHouse,
                [1, 1, 3] => HandType::ThreeOfAKind,
                [1, 2, 2] => HandType::TwoPair,
                [1, 1, 1, 2] => HandType::OnePair,
                [1, 1, 1, 1, 1] => HandType::HighCard,
                _ => panic!()
            }
    }
}

impl From<&str> for Hand {
    fn from(value: &str) -> Self {
        assert_eq!(value.len(), 5);
        Hand(
            value
                .chars()
                .map_into::<Card>()
                .collect_vec()
                .try_into()
                .unwrap(),
        )
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match HandType::from(self).cmp(&HandType::from(other)) {
            Ordering::Equal => self.0.cmp(&other.0),
            x => x,
        }
    }
}

fn parse_hand(line: &str) -> (Hand, usize) {
    let (h, bid) = line.split_once(' ').unwrap();
    (Hand::from(h), bid.parse().unwrap())
}

#[aoc(day7, part1)]
pub fn part_1(input: &str) -> usize {
    input
        .lines()
        .map(parse_hand)
        .sorted()
        .enumerate()
        .map(|(i, (_, bid))| (i + 1) * bid)
        .sum()
}

// #[aoc(day7, part2)]
// pub fn part_2(input: &str) -> i32 {
//     input.lines().map(clean_up_line_part_2).sum()
// }

#[cfg(test)]
mod test {
    // use crate::day1::clean_up_line_part_2;

    // #[test]
    // fn test_consume_strings() {
    //     assert_eq!(clean_up_line_part_2("two1nine"), 29)
    // }
}
