use std::collections::HashMap;

fn read_graph_entry(line: &str) -> (String, (String, String)) {
    let (start, end) = line.split_once(" = ").unwrap();

    let simplified_end = end.replace(['(', ')'], "");

    let (left, right) = simplified_end.split_once(", ").unwrap();

    (start.to_owned(), (left.to_owned(), right.to_owned()))
}

#[aoc(day8, part1)]
pub fn part_1(input: &str) -> i64 {
    let mut lines = input.lines();
    let directions = lines.next().unwrap();

    // Discard empty line
    lines.next().unwrap();

    let mut graph: HashMap<String, (String, String)> = HashMap::new();
    for line in lines {
        let (k, v) = read_graph_entry(line);
        graph.insert(k, v);
    }

    let mut position = "AAA".to_owned();
    let mut steps = 0;

    loop {
        for direction in directions.chars() {
            position = match direction {
                'L' => graph[&position].0.clone(),
                'R' => graph[&position].1.clone(),
                _ => panic!(),
            };
            steps += 1;
            if position == "ZZZ" {
                return steps;
            }
        }
    }
}

fn get_num_steps_to_end(
    graph: &HashMap<String, (String, String)>,
    directions: &str,
    start: &str,
) -> i64 {
    let mut position = start.to_owned();

    let mut steps = 0;
    loop {
        for direction in directions.chars() {
            position = match direction {
                'L' => graph[&position].0.clone(),
                'R' => graph[&position].1.clone(),
                _ => panic!(),
            };
            steps += 1;
            if position.ends_with('Z') {
                return steps;
            }
        }
    }
}

#[aoc(day8, part2)]
pub fn part_2(input: &str) -> i64 {
    let mut lines = input.lines();
    let directions = lines.next().unwrap();

    // Discard empty line
    lines.next().unwrap();

    let mut graph: HashMap<String, (String, String)> = HashMap::new();
    let mut starting_positions = vec![];
    for line in lines {
        let (k, v) = read_graph_entry(line);
        graph.insert(k.clone(), v);
        if k.ends_with('A') {
            starting_positions.push(k);
        }
    }

    starting_positions
        .iter()
        .map(|pos| get_num_steps_to_end(&graph, directions, pos))
        .reduce(num::integer::lcm)
        .unwrap()
}

#[cfg(test)]
mod test {
    use crate::day08::part_2;

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "LR\n\
\n\
                11A = (11B, XXX)\n\
                11B = (XXX, 11Z)\n\
                11Z = (11B, XXX)\n\
                22A = (22B, XXX)\n\
                22B = (22C, 22C)\n\
                22C = (22Z, 22Z)\n\
                22Z = (22B, 22B)\n\
                XXX = (XXX, XXX)"
            ),
            6
        )
    }
}
