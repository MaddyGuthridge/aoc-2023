use std::{cmp::Ordering, collections::HashMap};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Card {
    J,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
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
            x => panic!("Unknown card type {x}"),
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

impl From<&Hand> for HandType {
    fn from(value: &Hand) -> Self {
        let mut card_counts: HashMap<Card, u8> = HashMap::default();

        let mut num_jokers = 0;

        for card in value.0 {
            if let Card::J = card {
                num_jokers += 1;
            }
            card_counts.insert(card, card_counts.get(&card).unwrap_or(&0) + 1);
        }

        let mut num_pairs = 0;
        let mut num_triples = 0;

        for (card, count) in card_counts {
            match count {
                1 => {
                    if num_jokers == 4 {
                        return HandType::FiveOfAKind;
                    }
                }
                2 => {
                    if num_jokers == 3 {
                        return HandType::FiveOfAKind;
                    }
                    if num_jokers == 2 && card != Card::J {
                        return HandType::FourOfAKind;
                    }
                    num_pairs += 1;
                }
                3 => {
                    if num_jokers == 2 {
                        return HandType::FiveOfAKind;
                    }
                    if num_jokers == 1 {
                        return HandType::FourOfAKind;
                    }
                    num_triples += 1;
                }
                4 => {
                    if num_jokers == 1 || num_jokers == 4 {
                        return HandType::FiveOfAKind;
                    }
                    return HandType::FourOfAKind;
                }
                5 => {
                    return HandType::FiveOfAKind;
                }
                _ => {}
            }
        }

        if num_pairs == 2 {
            match num_jokers {
                1 => HandType::FullHouse,
                2 => HandType::FourOfAKind,
                0 => HandType::TwoPair,
                _ => panic!(),
            }
        } else if num_pairs == 1 {
            if num_triples == 1 {
                HandType::FullHouse
            } else {
                match num_jokers {
                    0 => HandType::OnePair,
                    1 => HandType::ThreeOfAKind,
                    // 2 jokers are the only pair - match them with another random card
                    2 => HandType::ThreeOfAKind,
                    3 => panic!("Should be five of a kind"),
                    _ => panic!(),
                }
            }
        } else if num_triples == 1 {
            if num_jokers == 3 {
                HandType::FourOfAKind
            } else {
                HandType::ThreeOfAKind
            }
        } else if num_jokers == 1 {
            HandType::OnePair
        } else {
            HandType::HighCard
        }
    }
}

#[derive(Debug)]
struct Hand([Card; 5]);

impl From<&str> for Hand {
    fn from(value: &str) -> Self {
        assert_eq!(value.len(), 5);
        Hand(
            value
                .chars()
                .map_into::<Card>()
                .collect_vec()
                .try_into()
                .expect("Vec of wrong size?"),
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

#[aoc(day7, part2)]
pub fn part_2(input: &str) -> usize {
    input
        .lines()
        .map(parse_hand)
        .sorted()
        .enumerate()
        .map(|(i, (_, bid))| (i + 1) * bid)
        .sum()
}

#[cfg(test)]
mod test {
    use crate::day07_part_2::{part_2, Hand, HandType};

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "32T3K 765\n\
                T55J5 684\n\
                KK677 28\n\
                KTJJT 220\n\
                QQQJA 483"
            ),
            5905,
        )
    }

    #[test]
    fn get_hand_type_five() {
        assert_eq!(HandType::from(&Hand::from("QJJJJ")), HandType::FiveOfAKind);
        assert_eq!(HandType::from(&Hand::from("QQJJJ")), HandType::FiveOfAKind);
        assert_eq!(HandType::from(&Hand::from("QQQJJ")), HandType::FiveOfAKind);
        assert_eq!(HandType::from(&Hand::from("QQQQJ")), HandType::FiveOfAKind);
    }

    #[test]
    fn get_hand_type_four() {
        assert_eq!(HandType::from(&Hand::from("QJJJA")), HandType::FourOfAKind);
        assert_eq!(HandType::from(&Hand::from("QQJJA")), HandType::FourOfAKind);
        assert_eq!(HandType::from(&Hand::from("QQQJA")), HandType::FourOfAKind);
    }

    #[test]
    fn get_hand_type_full_house() {
        assert_eq!(HandType::from(&Hand::from("QQAAJ")), HandType::FullHouse);
    }

    #[test]
    fn get_hand_type_three() {
        assert_eq!(HandType::from(&Hand::from("QQJ32")), HandType::ThreeOfAKind)
    }

    #[test]
    fn sort_cards() {
        assert!(Hand::from("JQQAA") < Hand::from("QQQAA"));
        assert!(Hand::from("QJQAA") < Hand::from("QQQAA"));
        assert!(Hand::from("JKKK2") < Hand::from("QQQQ2"));
        assert!(Hand::from("J2345") < Hand::from("22456"));
    }
}
