use itertools::Itertools;

#[derive(Debug)]
enum SymbolType {
    Normal,
    Gear,
}

#[derive(Debug)]
enum Cell {
    Empty,
    Symbol(SymbolType),
    Number(usize),
}

impl Cell {
    fn is_number(&self) -> bool {
        matches!(self, Cell::Number(_))
    }
}

fn exists_in_grid<T>(cells: &Vec<Vec<T>>, r: i32, c: i32) -> bool {
    0 <= r && 0 <= c && r < cells.len() as i32 && c < cells[0].len() as i32
}

fn line_to_cell_line(line: &str) -> Vec<Cell> {
    line.chars()
        .map(|c| match c {
            '.' => Cell::Empty,
            _ => {
                if c.is_ascii_digit() {
                    Cell::Number(c.to_digit(10).unwrap() as usize)
                } else if c == '*' {
                    Cell::Symbol(SymbolType::Gear)
                } else {
                    Cell::Symbol(SymbolType::Normal)
                }
            }
        })
        .collect()
}

/// only mark numbers
fn recursive_mark(cells: &Vec<Vec<Cell>>, valid: &mut Vec<Vec<bool>>, r: i32, c: i32) {
    if !exists_in_grid(cells, r, c) {
        return;
    }
    if valid[r as usize][c as usize] {
        return;
    }
    if !cells[r as usize][c as usize].is_number() {
        return;
    }

    valid[r as usize][c as usize] = true;

    for off_c in -1..=1 {
        recursive_mark(cells, valid, r, c + off_c);
    }
}

fn mark_valid_numbers(cells: &Vec<Vec<Cell>>, valid: &mut Vec<Vec<bool>>) {
    for (r, row_data) in cells.iter().enumerate() {
        for (c, col_data) in row_data.iter().enumerate() {
            if matches!(col_data, Cell::Symbol(_)) {
                for off_r in -1..=1 {
                    for off_c in -1..=1 {
                        recursive_mark(cells, valid, r as i32 + off_r, c as i32 + off_c);
                    }
                }
            }
        }
    }
}

fn calc_sum(cells: &Vec<Vec<Cell>>, valid: &Vec<Vec<bool>>) -> usize {
    let mut sum = 0;
    for (r, row_data) in cells.iter().enumerate() {
        let mut curr_sum: Option<usize> = None;
        for (c, col_data) in row_data.iter().enumerate() {
            if valid[r][c] {
                let mut temp_sum = curr_sum.unwrap_or(0);
                if let Cell::Number(n) = col_data {
                    temp_sum = n + 10 * temp_sum;
                }
                curr_sum = Some(temp_sum);
            } else {
                if let Some(csum) = curr_sum {
                    sum += csum;
                }
                curr_sum = None;
            }
        }
        if let Some(n) = curr_sum {
            sum += n;
        }
    }
    sum
}

fn calc_number(cells: &Vec<Vec<Cell>>, r: usize, c: usize) -> usize {
    if 1 <= c && matches!(cells[r][c - 1], Cell::Number(_)) {
        return calc_number(cells, r, c - 1);
    }

    // This is the first number in the sequence, so let's iterate
    let mut num = 0;

    for cell in &cells[r][c..] {
        if let Cell::Number(n) = cell {
            num = num * 10 + n;
        } else {
            return num;
        }
    }

    num
}

fn calc_gear_ratio(cells: &Vec<Vec<Cell>>) -> usize {
    let mut sum = 0;

    for (r, row_data) in cells.iter().enumerate() {
        for (c, col_data) in row_data.iter().enumerate() {
            if matches!(col_data, Cell::Symbol(SymbolType::Gear)) {
                // Now check for surrounding numbers
                let mut surround_mul = 1;
                let mut found_nums = 0;

                for off_r in -1..=1 {
                    let mut saw_num_last = false;
                    for off_c in -1..=1 {
                        if !exists_in_grid(cells, r as i32 + off_r, c as i32 + off_c) {
                            continue;
                        }
                        if matches!(
                            cells[(r as i32 + off_r) as usize][(c as i32 + off_c) as usize],
                            Cell::Number(_)
                        ) {
                            if saw_num_last {
                                continue;
                            }
                            surround_mul *= calc_number(
                                cells,
                                (r as i32 + off_r) as usize,
                                (c as i32 + off_c) as usize,
                            );
                            found_nums += 1;
                            saw_num_last = true;
                        } else {
                            saw_num_last = false;
                        }
                    }
                }

                // Only valid if exactly 2 numbers next to the gear
                if found_nums == 2 {
                    sum += surround_mul;
                }
            }
        }
    }

    sum
}

#[aoc(day3, part1)]
pub fn part_1(input: &str) -> usize {
    let cells = input.lines().map(line_to_cell_line).collect_vec();

    let mut valid_cells = vec![vec![false; cells[0].len()]; cells.len()];

    mark_valid_numbers(&cells, &mut valid_cells);

    calc_sum(&cells, &valid_cells)
}

#[aoc(day3, part2)]
pub fn part_2(input: &str) -> usize {
    let cells = input.lines().map(line_to_cell_line).collect_vec();

    calc_gear_ratio(&cells)
}

#[cfg(test)]
mod test {
    use crate::day3::{calc_number, part_1, Cell, part_2};

    #[test]
    fn test_simple() {
        assert_eq!(part_1(".1.\n.*."), 1);
    }

    #[test]
    fn test_advanced() {
        assert_eq!(
            part_1(
                "467..114..\n\
                ...*......\n\
                ..35..633.\n\
                ......#...\n\
                617*......\n\
                .....+.58.\n\
                ..592.....\n\
                ......755.\n\
                ...$.*....\n\
                .664.598.."
            ),
            4361
        )
    }

    #[test]
    fn test_numbers_touch() {
        assert_eq!(
            part_1(
                "+....\n\
                 12...\n\
                 34..."
            ),
            12
        )
    }

    #[test]
    fn test_find_number() {
        assert_eq!(calc_number(&vec![vec![Cell::Number(3), Cell::Number(2), Cell::Number(1)]], 0, 0), 321);
        assert_eq!(calc_number(&vec![vec![Cell::Number(3), Cell::Number(2), Cell::Number(1)]], 0, 2), 321);
    }

    #[test]
    fn test_calc_gear_ratio() {
        assert_eq!(
            part_2(
                "467..114..\n\
                ...*......\n\
                ..35..633.\n\
                ......#...\n\
                617*......\n\
                .....+.58.\n\
                ..592.....\n\
                ......755.\n\
                ...$.*....\n\
                .664.598.."
            ),
            467835
        )
    }
}
