use array2d::Array2D;
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
enum Pixel {
    Galaxy,
    Empty,
}

impl Pixel {
    fn is_galaxy(&self) -> bool {
        matches!(self, Pixel::Galaxy)
    }
}

impl From<char> for Pixel {
    fn from(value: char) -> Self {
        match value {
            '#' => Pixel::Galaxy,
            '.' => Pixel::Empty,
            _ => panic!("Invalid pixel"),
        }
    }
}

fn parse_image(input: &str) -> Array2D<Pixel> {
    Array2D::from_rows(
        &input
            .lines()
            .map(|line| line.chars().map(Pixel::from).collect_vec())
            .collect_vec(),
    )
    .unwrap()
}

fn find_empty_rows_cols(image: &Array2D<Pixel>) -> (Vec<usize>, Vec<usize>) {
    let empty_rows = image
        .rows_iter()
        .enumerate()
        .filter_map(|(i, mut row)| {
            if row.any(|c| c.is_galaxy()) {
                None
            } else {
                Some(i)
            }
        })
        .collect_vec();
    let empty_cols = image
        .columns_iter()
        .enumerate()
        .filter_map(|(i, mut col)| {
            if col.any(|c| c.is_galaxy()) {
                None
            } else {
                Some(i)
            }
        })
        .collect_vec();

    (empty_rows, empty_cols)
}

fn find_pois(image: &Array2D<Pixel>) -> Vec<(usize, usize)> {
    image
        .enumerate_row_major()
        .filter_map(|(p, cell)| if cell.is_galaxy() { Some(p) } else { None })
        .collect_vec()
}

fn expand_pois(
    pois: &[(usize, usize)],
    empty_rows: &[usize],
    empty_cols: &[usize],
    expansion_factor: usize,
) -> Vec<(usize, usize)> {
    pois.iter()
        .map(|(r, c)| {
            let new_r =
                r + empty_rows.iter().filter(|empty_row| *empty_row < r).count() * (expansion_factor - 1);
            let new_c =
                c + empty_cols.iter().filter(|empty_col| *empty_col < c).count() * (expansion_factor - 1);
            (new_r, new_c)
        })
        .collect_vec()
}

fn calculate_distance(g1: &(usize, usize), g2: &(usize, usize)) -> usize {
    g1.0.abs_diff(g2.0) + g1.1.abs_diff(g2.1)
}

fn calc_total_distance(galaxy_positions: &[(usize, usize)]) -> usize {
    galaxy_positions
        .iter()
        .combinations(2)
        .map(|galaxies| {
            let (g1, g2) = (galaxies[0], galaxies[1]);
            calculate_distance(g1, g2)
        })
        .sum()
}

#[aoc(day11, part1)]
pub fn part_1(input: &str) -> usize {
    let image = parse_image(input);

    let (empty_rows, empty_cols) = find_empty_rows_cols(&image);

    let galaxy_positions = expand_pois(&find_pois(&image), &empty_rows, &empty_cols, 2);

    calc_total_distance(&galaxy_positions)
}

#[aoc(day11, part2)]
pub fn part_2(input: &str) -> usize {
    let image = parse_image(input);

    let (empty_rows, empty_cols) = find_empty_rows_cols(&image);

    let galaxy_positions = expand_pois(&find_pois(&image), &empty_rows, &empty_cols, 1_000_000);

    calc_total_distance(&galaxy_positions)
}

#[cfg(test)]
mod test {
    use super::part_1;

    #[test]
    fn test_calc_distances() {
        assert_eq!(
            part_1(
                "#.\n\
                 .#"
            ),
            2
        )
    }

    #[test]
    fn test_calc_distances_with_expansion() {
        assert_eq!(
            part_1(
                "#..\n\
                 ..#"
            ),
            4
        )
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "...#......\n\
                 .......#..\n\
                 #.........\n\
                 ..........\n\
                 ......#...\n\
                 .#........\n\
                 .........#\n\
                 ..........\n\
                 .......#..\n\
                 #...#....."
            ),
            374
        )
    }
}
