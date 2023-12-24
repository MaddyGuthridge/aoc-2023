use std::{
    fmt::Debug,
    ops::{Index, IndexMut, Range}, collections::HashSet,
};

use array2d::Array2D;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Brick {
    z1: usize,
    z2: usize,
    x1: usize,
    x2: usize,
    y1: usize,
    y2: usize,
    id: usize,
}

impl Brick {
    fn new(value: &str, id: usize) -> Self {
        let (xyz1, xyz2) = value.split_once('~').unwrap();
        let (x1, y1, z1): (usize, usize, usize) = xyz1
            .split(',')
            .map(|n| n.parse().unwrap())
            .collect_tuple()
            .unwrap();
        let (x2, y2, z2): (usize, usize, usize) = xyz2
            .split(',')
            .map(|n| n.parse().unwrap())
            .collect_tuple()
            .unwrap();

        Brick {
            id,
            // Reduce z values by 1 to save space
            z1: z1 - 1,
            z2: z2 - 1,
            x1,
            x2,
            y1,
            y2,
        }
    }
}

#[derive(Clone)]
struct BrickPile {
    /// Space that the bricks are in
    ///
    /// Each vec is some horizontal space
    /// Then each 2D array is a horizontal place
    ///
    /// Elements are the indexes of bricks in the bricks vec
    space: Vec<Array2D<Option<usize>>>,
    /// Vec containing all bricks
    bricks: Vec<Brick>,
}

impl Index<(usize, usize, usize)> for BrickPile {
    type Output = Option<usize>;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.space[index.2][(index.0, index.1)]
    }
}

impl IndexMut<(usize, usize, usize)> for BrickPile {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.space[index.2][(index.0, index.1)]
    }
}

impl Index<usize> for BrickPile {
    type Output = Brick;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bricks[index]
    }
}

impl IndexMut<usize> for BrickPile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bricks[index]
    }
}

impl Debug for BrickPile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, layer) in self.space.iter().enumerate() {
            writeln!(f, "Layer {i}")?;
            for row in layer.rows_iter() {
                for val in row {
                    if let Some(v) = val {
                        write!(f, "{v}")?;
                    } else {
                        write!(f, ".")?;
                    }
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl BrickPile {
    fn new(bricks: Vec<Brick>) -> Self {
        let (x, y, z) = bricks
            .iter()
            .fold((1, 1, 1), |(mut x, mut y, mut z), curr| {
                if x <= curr.x2 {
                    x = curr.x2 + 1;
                }
                if y <= curr.y2 {
                    y = curr.y2 + 1;
                }
                if z <= curr.z2 {
                    z = curr.z2 + 1;
                }
                (x, y, z)
            });

        let mut pile = BrickPile {
            space: vec![Array2D::from_rows(&vec![vec![None; x]; y]).unwrap(); z],
            bricks: bricks.clone(),
        };

        // Fill each brick
        for (i, brick) in bricks.iter().enumerate() {
            pile.fill(
                (brick.x1, brick.x2),
                (brick.y1, brick.y2),
                (brick.z1, brick.z2),
                Some(i),
            );
        }

        // Now make all the bricks fall into place
        pile.stabilise();

        pile
    }

    fn brick_indexes(&self) -> Range<usize> {
        0..self.bricks.len()
    }

    /// Uses inclusive ranges
    fn fill(
        &mut self,
        (x1, x2): (usize, usize),
        (y1, y2): (usize, usize),
        (z1, z2): (usize, usize),
        value: Option<usize>,
    ) {
        for x in x1..=x2 {
            for y in y1..=y2 {
                for z in z1..=z2 {
                    self[(x, y, z)] = value;
                }
            }
        }
    }

    fn find_farthest_empty_layer(&self, brick_index: usize) -> Option<usize> {
        let brick = &self[brick_index];
        // Check each layer
        for z in (0..brick.z1).rev() {
            // For this layer, make sure all space is empty
            for x in brick.x1..=brick.x2 {
                for y in brick.y1..=brick.y2 {
                    if self[(x, y, z)].is_some() {
                        return Some(z + 1);
                    }
                }
            }
        }
        // Reached the bottom
        Some(0)
    }

    fn make_brick_fall(&mut self, brick_index: usize) {
        let layer = self.find_farthest_empty_layer(brick_index);

        let brick = self[brick_index].clone();

        if layer.is_none() {
            return;
        }
        let layer = layer.unwrap();
        let height = brick.z2 - brick.z1;

        // Remove brick from the pile
        self.fill(
            (brick.x1, brick.x2),
            (brick.y1, brick.y2),
            (brick.z1, brick.z2),
            None,
        );

        // Then update the brick position
        self[brick_index].z1 = layer;
        self[brick_index].z2 = layer + height;

        let brick = self[brick_index].clone();

        // And add it in at the new location
        self.fill(
            (brick.x1, brick.x2),
            (brick.y1, brick.y2),
            (brick.z1, brick.z2),
            Some(brick_index),
        );
        self.fill(
            (brick.x1, brick.x2),
            (brick.y1, brick.y2),
            (brick.z1, brick.z2),
            Some(brick_index),
        );
    }

    /// Make all bricks fall until they come to rest
    fn stabilise(&mut self) {
        // Since all the bricks are sorted by height - we can make them fall
        // one by one and it'll work
        for b in self.bricks.iter().sorted().cloned().collect_vec() {
            self.make_brick_fall(b.id);
        }
    }

    fn find_bricks_in_region(
        &self,
        (x1, x2): (usize, usize),
        (y1, y2): (usize, usize),
        z: usize,
    ) -> Vec<usize> {
        let mut supports = vec![];

        // Check each region beneath the brick
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(support_index) = self[(x, y, z)] {
                    if !supports.contains(&support_index) {
                        supports.push(support_index);
                    }
                }
            }
        }

        supports
    }

    /// Return the indexes of all bricks supporting this brick
    fn find_supports(&self, brick_index: usize) -> Vec<usize> {
        let brick = &self[brick_index];
        let layer = self[brick_index].z1;
        if layer == 0 {
            return vec![];
        }
        let z = layer - 1;

        self.find_bricks_in_region((brick.x1, brick.x2), (brick.y1, brick.y2), z)
    }

    /// Return the indexes of all bricks this brick supports
    fn find_supporting(&self, brick_index: usize) -> Vec<usize> {
        let brick = &self[brick_index];
        let layer = self[brick_index].z2;
        if layer == self.space.len() - 1 {
            return vec![];
        }
        let z = layer + 1;

        self.find_bricks_in_region((brick.x1, brick.x2), (brick.y1, brick.y2), z)
    }

    fn find_supporting_recursive(
        &mut self,
        brick_index: usize,
    ) -> HashSet<usize> {
        let mut res = self.do_find_supporting_recursive(brick_index, vec![brick_index]);

        res.remove(&brick_index);

        res
    }

    /// Return the indexes of all bricks this brick supports, including
    /// indirectly
    fn do_find_supporting_recursive(
        &mut self,
        brick_index: usize,
        lower_supports: Vec<usize>,
    ) -> HashSet<usize> {
        let bricks_we_will_destroy = self
            .find_supporting(brick_index)
            .into_iter()
            // Only include supported bricks if we're the only support
            .filter(|&b| {
                let supports = self.find_supports(b);
                supports
                    .into_iter()
                    // All supporting bricks for the current brick are ones
                    // that the original brick directly supports
                    .all(|support| lower_supports.contains(&support))
            })
            // Collect into a vec first to keep the borrow checker happy
            // Otherwise, we're reading from and writing to the cache at
            // the same time which is a big no no!
            .collect_vec();

        // Cache miss, calculate value
        let ret = bricks_we_will_destroy
            .iter()
            // And for each one, expand it into the values that it supports
            .flat_map(|&b| {
                // Bricks that are in our "tower of supported bricks"
                let curr_lower_supports = lower_supports
                    .iter()
                    .chain(bricks_we_will_destroy.iter())
                    .copied()
                    .collect_vec();
                let mut res = self.do_find_supporting_recursive(b, curr_lower_supports);
                // Don't forget to include this one too
                res.insert(b);
                res
            })
            .collect();

        ret
    }
}

#[aoc(day22, part1)]
pub fn part_1(input: &str) -> usize {
    let bricks: Vec<Brick> = input
        .lines()
        .enumerate()
        .map(|(i, b)| Brick::new(b, i))
        .collect_vec();

    let pile = BrickPile::new(bricks);

    // Now find all the bricks that are only supporting bricks that have at least 2 supports
    pile.brick_indexes()
        .filter(|&b| {
            // dbg!(b);
            !pile
                .find_supporting(b)
                .into_iter()
                .any(|b| pile.find_supports(b).len() == 1)
        })
        .count()
}

#[aoc(day22, part2)]
pub fn part_2(input: &str) -> usize {
    let bricks: Vec<Brick> = input
        .lines()
        .enumerate()
        .map(|(i, b)| Brick::new(b, i))
        .collect_vec();

    let mut pile = BrickPile::new(bricks);

    // dbg!(&pile);

    pile.brick_indexes()
        .map(|b| pile.find_supporting_recursive(b).len())
        .sum()
}

#[cfg(test)]
mod test {
    use super::{part_1, part_2};

    #[test]
    fn test_simple() {
        assert_eq!(
            part_1(
                "0,0,1~0,0,1\n\
                0,0,2~0,0,2"
            ),
            1
        )
    }

    #[test]
    fn test_fall_into_slot() {
        assert_eq!(
            part_1(
                "0,0,1~2,0,1\n\
                0,2,1~2,2,1\n\
                0,1,2~2,1,2\n\
                1,0,3~1,2,3"
            ),
            4
        )
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "1,0,1~1,2,1\n\
                0,0,2~2,0,2\n\
                0,2,3~2,2,3\n\
                0,0,4~0,2,4\n\
                2,0,5~2,2,5\n\
                0,1,6~2,1,6\n\
                1,1,8~1,1,9"
            ),
            5,
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "1,0,1~1,2,1\n\
                0,0,2~2,0,2\n\
                0,2,3~2,2,3\n\
                0,0,4~0,2,4\n\
                2,0,5~2,2,5\n\
                0,1,6~2,1,6\n\
                1,1,8~1,1,9"
            ),
            7,
        )
    }

    #[test]
    fn test_simple_part_2() {
        assert_eq!(
            part_2(
                "0,0,1~0,0,1\n\
                0,0,2~0,0,2\n\
                0,0,3~0,0,3"
            ),
            3
        )
    }

    // FIXME: For some reason, the answer is too low
    // Hunt for a case where bricks that would fall are missed
}
