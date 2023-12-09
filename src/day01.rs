use itertools::{EitherOrBoth::*, Itertools};

fn clean_up_line_part_1(line: &str) -> i32 {
    let mut first: Option<char> = None;
    let mut last: Option<char> = None;

    for c in line.chars() {
        if c.is_ascii_digit() {
            if first.is_none() {
                first = Some(c);
            }
            last = Some(c);
        }
    }

    format!("{}{}", first.unwrap(), last.unwrap())
        .parse()
        .unwrap()
}

#[aoc(day1, part1)]
pub fn part_1(input: &str) -> i32 {
    input.lines().map(clean_up_line_part_1).sum()
}

fn try_to_digit(chars: &mut dyn Iterator<Item = char>, expected: &str, give: u32) -> Option<u32> {
    for pair in expected.chars().zip_longest(chars) {
        match pair {
            Both(l, r) => {
                if l != r {
                    return None;
                }
            }
            Left(_) => {
                return None;
            }
            Right(_) => {
                return Some(give);
            }
        }
    }
    Some(give)
}

fn process_chars(chars: &str) -> Option<u32> {
    let mut chars_iterator = chars.chars();
    let first_letter = chars_iterator.next()?;
    match first_letter {
        'o' => try_to_digit(&mut chars_iterator, "ne", 1),
        't' => match chars_iterator.next()? {
            'w' => try_to_digit(&mut chars_iterator, "o", 2),
            'h' => try_to_digit(&mut chars_iterator, "ree", 3),
            _ => None,
        },
        'f' => match chars_iterator.next()? {
            'o' => try_to_digit(&mut chars_iterator, "ur", 4),
            'i' => try_to_digit(&mut chars_iterator, "ve", 5),
            _ => None,
        },
        's' => match chars_iterator.next()? {
            'i' => try_to_digit(&mut chars_iterator, "x", 6),
            'e' => try_to_digit(&mut chars_iterator, "ven", 7),
            _ => None,
        },
        'e' => try_to_digit(&mut chars_iterator, "ight", 8),
        'n' => try_to_digit(&mut chars_iterator, "ine", 9),
        _ => {
            if first_letter.is_ascii_digit() {
                Some(first_letter.to_digit(10).unwrap())
            } else {
                None
            }
        }
    }
}

fn clean_up_line_part_2(line: &str) -> i32 {
    let mut first: Option<u32> = None;
    let mut last: Option<u32> = None;

    for (c_index, _) in line.char_indices() {
        if let Some(num) = process_chars(&line[c_index..]) {
            if first.is_none() {
                first = Some(num);
            }
            last = Some(num);
        }
    }

    format!("{}{}", first.unwrap(), last.unwrap())
        .parse()
        .unwrap()
}

#[aoc(day1, part2)]
pub fn part_2(input: &str) -> i32 {
    input.lines().map(clean_up_line_part_2).sum()
}

#[cfg(test)]
mod test {
    use crate::day01::clean_up_line_part_2;

    #[test]
    fn test_consume_strings() {
        assert_eq!(clean_up_line_part_2("two1nine"), 29)
    }
}
