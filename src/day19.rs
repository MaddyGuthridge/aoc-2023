use std::collections::HashMap;

use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
enum Operator {
    Gt,
    Lt,
}

impl From<char> for Operator {
    fn from(value: char) -> Self {
        match value {
            '>' => Operator::Gt,
            '<' => Operator::Lt,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Property {
    X,
    M,
    A,
    S,
}

impl From<char> for Property {
    fn from(value: char) -> Self {
        match value {
            'x' => Property::X,
            'm' => Property::M,
            'a' => Property::A,
            's' => Property::S,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
struct Condition {
    prop: Property,
    op: Operator,
    compare_to: usize,
    output_to: String,
}

impl From<&str> for Condition {
    fn from(value: &str) -> Self {
        let prop = Property::from(value.chars().next().unwrap());
        let op = Operator::from(value.chars().nth(1).unwrap());
        let (temp, output_to) = value.split_once(':').unwrap();
        let compare_to = temp
            .chars()
            .skip(2)
            .collect::<String>()
            .parse::<usize>()
            .unwrap();

        Self {
            prop,
            op,
            compare_to,
            output_to: output_to.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
struct Uncondition {
    output_to: String,
}

impl TryFrom<&str> for Uncondition {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains(':') {
            Err("Contained a :".to_owned())
        } else {
            Ok(Uncondition {
                output_to: value.to_owned(),
            })
        }
    }
}

#[derive(Debug, Clone)]
enum Rule {
    Condition(Condition),
    Uncondition(Uncondition),
}

impl Rule {
    fn get_output(&self) -> &str {
        match self {
            Rule::Condition(c) => &c.output_to,
            Rule::Uncondition(u) => &u.output_to,
        }
    }
}

impl From<&str> for Rule {
    fn from(value: &str) -> Self {
        if let Ok(unc) = Uncondition::try_from(value) {
            Rule::Uncondition(unc)
        } else {
            Rule::Condition(Condition::from(value))
        }
    }
}

#[derive(Debug, Clone)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
}

impl From<&str> for Workflow {
    fn from(value: &str) -> Self {
        let (name, rules) = value.split_once('{').unwrap();

        let rules: Vec<Rule> = rules.replace('}', "").split(',').map_into().collect_vec();

        Workflow {
            name: name.to_owned(),
            rules,
        }
    }
}

#[derive(Debug, Clone)]
struct Part {
    x: usize,
    m: usize,
    a: usize,
    s: usize,
}

impl From<&str> for Part {
    fn from(value: &str) -> Self {
        let temp = value.replace(['{', '}', '=', 'x', 'm', 'a', 's'], "");

        let (x, m, a, s) = temp.split(',').collect_tuple().unwrap();

        Part {
            x: x.parse().unwrap(),
            m: m.parse().unwrap(),
            a: a.parse().unwrap(),
            s: s.parse().unwrap(),
        }
    }
}

impl PartialEq<Condition> for Part {
    fn eq(&self, other: &Condition) -> bool {
        let prop_to_check = match other.prop {
            Property::X => self.x,
            Property::M => self.m,
            Property::A => self.a,
            Property::S => self.s,
        };

        match other.op {
            Operator::Gt => prop_to_check > other.compare_to,
            Operator::Lt => prop_to_check < other.compare_to,
        }
    }
}

impl PartialEq<Uncondition> for Part {
    fn eq(&self, _: &Uncondition) -> bool {
        true
    }
}

impl PartialEq<Rule> for Part {
    fn eq(&self, other: &Rule) -> bool {
        match other {
            Rule::Condition(c) => self == c,
            Rule::Uncondition(u) => self == u,
        }
    }
}

impl From<Part> for usize {
    fn from(value: Part) -> Self {
        value.x + value.m + value.a + value.s
    }
}

#[derive(Debug, Clone)]
struct PartRange {
    x: (usize, usize),
    m: (usize, usize),
    a: (usize, usize),
    s: (usize, usize),
}

impl From<PartRange> for usize {
    fn from(value: PartRange) -> Self {
        (value.x.1 - value.x.0 + 1)
            * (value.m.1 - value.m.0 + 1)
            * (value.a.1 - value.a.0 + 1)
            * (value.s.1 - value.s.0 + 1)
    }
}

impl Default for PartRange {
    fn default() -> Self {
        Self {
            x: (1, 4000),
            m: (1, 4000),
            a: (1, 4000),
            s: (1, 4000),
        }
    }
}

fn narrow_range_to(range: (usize, usize), op: Operator, val: usize) -> Option<(usize, usize)> {
    match op {
        Operator::Gt => {
            if range.0 > val {
                Some(range)
            } else if range.1 <= val {
                None
            } else {
                Some((val + 1, range.1))
            }
        }
        Operator::Lt => {
            if range.1 < val {
                Some(range)
            } else if range.0 >= val {
                None
            } else {
                Some((range.0, val - 1))
            }
        }
    }
}

fn narrow_range_against(range: (usize, usize), op: Operator, val: usize) -> Option<(usize, usize)> {
    match op {
        Operator::Gt => {
            if range.1 < val {
                Some(range)
            } else if range.0 >= val {
                None
            } else {
                Some((range.0, val))
            }
        }
        Operator::Lt => {
            if range.0 > val {
                Some(range)
            } else if range.1 <= val {
                None
            } else {
                Some((val, range.1))
            }
        }
    }
}

impl PartRange {
    fn narrow_to(&self, rule: &Rule) -> Option<PartRange> {
        let mut clone = self.clone();
        match rule {
            Rule::Uncondition(_) => Some(clone),
            Rule::Condition(c) => {
                match c.prop {
                    Property::X => {
                        clone.x = narrow_range_to(clone.x, c.op, c.compare_to)?;
                    }
                    Property::M => {
                        clone.m = narrow_range_to(clone.m, c.op, c.compare_to)?;
                    }
                    Property::A => {
                        clone.a = narrow_range_to(clone.a, c.op, c.compare_to)?;
                    }
                    Property::S => {
                        clone.s = narrow_range_to(clone.s, c.op, c.compare_to)?;
                    }
                }
                Some(clone)
            }
        }
    }

    fn narrow_against(&self, rule: &Rule) -> Option<PartRange> {
        let mut clone = self.clone();
        match rule {
            Rule::Uncondition(_) => None,
            Rule::Condition(c) => {
                match c.prop {
                    Property::X => {
                        clone.x = narrow_range_against(clone.x, c.op, c.compare_to)?;
                    }
                    Property::M => {
                        clone.m = narrow_range_against(clone.m, c.op, c.compare_to)?;
                    }
                    Property::A => {
                        clone.a = narrow_range_against(clone.a, c.op, c.compare_to)?;
                    }
                    Property::S => {
                        clone.s = narrow_range_against(clone.s, c.op, c.compare_to)?;
                    }
                }
                Some(clone)
            }
        }
    }
}

#[aoc(day19, part1)]
pub fn part_1(input: &str) -> usize {
    let (workflows, parts) = input.split_once("\n\n").unwrap();

    let workflows: HashMap<String, Vec<Rule>> = workflows
        .lines()
        .map_into::<Workflow>()
        .map(|w| (w.name, w.rules))
        .collect();

    parts
        .lines()
        .map_into::<Part>()
        .map(|part| {
            let mut curr_workflow = "in";
            while !["A", "R"].contains(&curr_workflow) {
                for rule in workflows.get(curr_workflow).unwrap() {
                    if part == *rule {
                        curr_workflow = rule.get_output();
                        break;
                    }
                }
            }

            if curr_workflow == "A" {
                usize::from(part)
            } else {
                0
            }
        })
        .sum()
}

fn determine_num_parts(
    workflows: &HashMap<String, Vec<Rule>>,
    curr_workflow: &str,
    part_range: &PartRange,
) -> usize {
    if curr_workflow == "A" {
        usize::from(part_range.clone())
    } else if curr_workflow == "R" {
        0
    } else {
        workflows
            .get(curr_workflow)
            .unwrap()
            .iter()
            .fold(
                (Some(part_range.clone()), 0),
                |(part_range, acc), rule| match part_range {
                    Some(range) => {
                        let narrowed_to = range.narrow_to(rule);
                        let num_parts_for_this_workflow = if let Some(narrowed) = narrowed_to {
                            determine_num_parts(workflows, rule.get_output(), &narrowed)
                        } else {
                            0
                        };
                        let narrowed_against = range.narrow_against(rule);
                        (narrowed_against, acc + num_parts_for_this_workflow)
                    }
                    None => (part_range, acc),
                },
            )
            .1
    }
}

#[aoc(day19, part2)]
pub fn part_2(input: &str) -> usize {
    let workflows: HashMap<String, Vec<Rule>> = input
        .split_once("\n\n")
        .unwrap()
        .0
        .lines()
        .map_into::<Workflow>()
        .map(|w| (w.name, w.rules))
        .collect();

    determine_num_parts(&workflows, "in", &PartRange::default())
}

#[cfg(test)]
mod test {
    use super::{narrow_range_against, narrow_range_to, part_1, part_2, Operator};

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "px{a<2006:qkq,m>2090:A,rfg}\n\
                pv{a>1716:R,A}\n\
                lnx{m>1548:A,A}\n\
                rfg{s<537:gd,x>2440:R,A}\n\
                qs{s>3448:A,lnx}\n\
                qkq{x<1416:A,crn}\n\
                crn{x>2662:A,R}\n\
                in{s<1351:px,qqz}\n\
                qqz{s>2770:qs,m<1801:hdj,R}\n\
                gd{a>3333:R,R}\n\
                hdj{m>838:A,pv}\n\
                \n\
                {x=787,m=2655,a=1222,s=2876}\n\
                {x=1679,m=44,a=2067,s=496}\n\
                {x=2036,m=264,a=79,s=2244}\n\
                {x=2461,m=1339,a=466,s=291}\n\
                {x=2127,m=1623,a=2188,s=1013}"
            ),
            19114
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "px{a<2006:qkq,m>2090:A,rfg}\n\
                pv{a>1716:R,A}\n\
                lnx{m>1548:A,A}\n\
                rfg{s<537:gd,x>2440:R,A}\n\
                qs{s>3448:A,lnx}\n\
                qkq{x<1416:A,crn}\n\
                crn{x>2662:A,R}\n\
                in{s<1351:px,qqz}\n\
                qqz{s>2770:qs,m<1801:hdj,R}\n\
                gd{a>3333:R,R}\n\
                hdj{m>838:A,pv}\n\
                \n\
                {x=787,m=2655,a=1222,s=2876}\n\
                {x=1679,m=44,a=2067,s=496}\n\
                {x=2036,m=264,a=79,s=2244}\n\
                {x=2461,m=1339,a=466,s=291}\n\
                {x=2127,m=1623,a=2188,s=1013}"
            ),
            167409079868000
        )
    }

    #[test]
    fn test_narrow_range_to() {
        // Gt
        assert_eq!(narrow_range_to((1, 4000), Operator::Gt, 0), Some((1, 4000)));
        assert_eq!(
            narrow_range_to((0, 4000), Operator::Gt, 100),
            Some((101, 4000))
        );
        assert_eq!(narrow_range_to((0, 4000), Operator::Gt, 4000), None);

        // Lt
        assert_eq!(
            narrow_range_to((0, 4000), Operator::Lt, 4001),
            Some((0, 4000))
        );
        assert_eq!(narrow_range_to((0, 4000), Operator::Lt, 100), Some((0, 99)));
        assert_eq!(narrow_range_to((0, 4000), Operator::Lt, 0), None);
    }

    #[test]
    fn test_narrow_range_against() {
        // Gt
        assert_eq!(
            narrow_range_against((0, 4000), Operator::Gt, 4000),
            Some((0, 4000))
        );
        assert_eq!(
            narrow_range_against((0, 4000), Operator::Gt, 100),
            Some((0, 100))
        );
        assert_eq!(narrow_range_against((1, 4000), Operator::Gt, 0), None);

        // Lt
        assert_eq!(
            narrow_range_against((0, 4000), Operator::Lt, 0),
            Some((0, 4000))
        );
        assert_eq!(
            narrow_range_against((0, 4000), Operator::Lt, 100),
            Some((100, 4000))
        );
        assert_eq!(narrow_range_against((0, 4000), Operator::Lt, 4001), None);
    }

    #[test]
    fn test_part_2_simple() {
        assert_eq!(
            part_2(
                "in{x<1000:R,A}\n\
                \n\
                {x=2127,m=1623,a=2188,s=1013}"
            ),
            4000 * 4000 * 4000 * 3001
        )
    }
}
