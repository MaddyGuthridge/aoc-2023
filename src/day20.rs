use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Mul, Not},
};

use itertools::Itertools;

type ModuleId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Pulse {
    High,
    Low,
}

impl Not for Pulse {
    type Output = Pulse;

    fn not(self) -> Self::Output {
        match self {
            Pulse::High => Pulse::Low,
            Pulse::Low => Pulse::High,
        }
    }
}

impl Display for Pulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match self {
            Pulse::High => "-high",
            Pulse::Low => "-low",
        };
        write!(f, "{disp}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PulseAtTime(usize, Pulse);

#[derive(Debug, Clone, Copy, Default)]
struct PulseCounter {
    high: usize,
    low: usize,
}

impl Add<Pulse> for PulseCounter {
    type Output = PulseCounter;

    fn add(mut self, rhs: Pulse) -> Self::Output {
        match rhs {
            Pulse::High => self.high += 1,
            Pulse::Low => self.low += 1,
        }
        self
    }
}

impl AddAssign<Pulse> for PulseCounter {
    fn add_assign(&mut self, rhs: Pulse) {
        match rhs {
            Pulse::High => self.high += 1,
            Pulse::Low => self.low += 1,
        }
    }
}

impl Add<PulseCounter> for PulseCounter {
    type Output = PulseCounter;

    fn add(self, rhs: PulseCounter) -> Self::Output {
        PulseCounter {
            high: self.high + rhs.high,
            low: self.low + rhs.low,
        }
    }
}

impl Sum for PulseCounter {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, curr| acc + curr).unwrap_or_default()
    }
}

impl Mul<usize> for PulseCounter {
    type Output = PulseCounter;

    fn mul(self, rhs: usize) -> Self::Output {
        PulseCounter {
            high: self.high * rhs,
            low: self.low * rhs,
        }
    }
}

impl From<PulseCounter> for usize {
    fn from(value: PulseCounter) -> Self {
        value.high * value.low
    }
}

#[derive(Debug, Clone)]
struct Event {
    from: ModuleId,
    target: ModuleId,
    pulse: Pulse,
}

#[derive(Debug, Clone, Default)]
struct EventQueue {
    queue: VecDeque<Event>,
}

impl EventQueue {
    fn push(&mut self, from: ModuleId, target: ModuleId, pulse: Pulse) {
        self.queue.push_back(Event {
            from,
            target,
            pulse,
        });
    }

    fn pop(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }

    // fn is_empty(&self) -> bool {
    //     self.queue.is_empty()
    // }

    fn drain(&mut self, modules: &mut [Module]) {
        while let Some(e) = self.pop() {
            // println!("{} {}-> {}", modules[e.from], e.pulse, modules[e.target]);
            modules[e.target].receive_pulse(e.from, e.pulse, self);
        }
    }
}

trait ModuleTrait: Debug {
    /// Receive a pulse from another node
    fn receive_pulse(&mut self, input: ModuleId, pulse: Pulse, event_queue: &mut EventQueue);

    /// Return a slice containing all the output IDs
    fn get_outputs(&self) -> &[ModuleId];

    /// Return whether the module is in its initial state, meaning it will
    /// produce the same pulse if given the same input as it was the first time
    /// around
    fn is_in_initial_state(&self) -> bool;

    /// Register an input module for this module
    fn register_input_module(&mut self, module: ModuleId);

    /// Return the number of button presses for this to pulse in the given
    /// state.
    ///
    /// Returns a tuple containing a vec of times and pulses, and a usize to represent the
    /// period of the cycle
    ///
    /// For example, if a module gives a low pulse every third button press, this
    /// would return `(vec![(2, Pulse::Low)], 3)`, representing a period of 3,
    /// for which, it triggers a low pulse on press 3 (index 2) each repetition
    fn get_presses_required_for_pulse(
        &self,
        other_modules: &[Module],
        visited_stack: &mut VecDeque<String>,
    ) -> (Vec<PulseAtTime>, usize);
}

/// Merge a vec of periods into a single period that contains all the pulses
fn merge_periods(periods: Vec<(Vec<PulseAtTime>, usize)>) -> (Vec<PulseAtTime>, usize) {
    // Calculate the total period
    let full_period = periods
        .iter()
        .map(|(_, p)| *p)
        .reduce(num::integer::lcm)
        .unwrap();

    // Now for each input, extend its pulses to be the length of the
    // full period, then flatten them all into one sorted array
    let full_period_data = periods
        .into_iter()
        .flat_map(|(pulses, period)| pulses.repeat(full_period / period))
        .sorted()
        .collect_vec();
    (full_period_data, full_period)
}

#[derive(Debug, Clone)]
struct Broadcaster {
    id: ModuleId,
    outputs: Vec<ModuleId>,
    inputs: Vec<ModuleId>,
}

impl Broadcaster {
    fn new(id: ModuleId, outputs: Vec<ModuleId>) -> Self {
        Broadcaster {
            id,
            outputs,
            inputs: vec![],
        }
    }
}

impl ModuleTrait for Broadcaster {
    fn receive_pulse(&mut self, _: ModuleId, pulse: Pulse, event_queue: &mut EventQueue) {
        for target in &self.outputs {
            event_queue.push(self.id, *target, pulse)
        }
    }

    fn is_in_initial_state(&self) -> bool {
        true
    }

    fn get_outputs(&self) -> &[ModuleId] {
        &self.outputs
    }

    fn register_input_module(&mut self, module: ModuleId) {
        self.inputs.push(module);
    }

    fn get_presses_required_for_pulse(
        &self,
        other_modules: &[Module],
        visited_stack: &mut VecDeque<String>,
    ) -> (Vec<PulseAtTime>, usize) {
        if self.inputs.is_empty() {
            (vec![PulseAtTime(0, Pulse::Low)], 1)
        } else {
            // Determine pulse times and periods of all parents
            // This may end up with a lot - I'm hoping that with only 58
            // modules in the input, it shouldn't cause too much chaos at least
            let input_data = self
                .inputs
                .iter()
                .map(|i| {
                    other_modules[*i].get_presses_required_for_pulse(other_modules, visited_stack)
                })
                .collect_vec();

            merge_periods(input_data)
        }
    }
}

#[derive(Debug, Clone)]
struct FlipFlop {
    id: ModuleId,
    state: Pulse,
    outputs: Vec<ModuleId>,
    inputs: Vec<ModuleId>,
}

impl FlipFlop {
    fn new(id: ModuleId, outputs: Vec<ModuleId>) -> Self {
        FlipFlop {
            id,
            state: Pulse::Low,
            outputs,
            inputs: vec![],
        }
    }
}

impl ModuleTrait for FlipFlop {
    fn receive_pulse(&mut self, _: ModuleId, pulse: Pulse, event_queue: &mut EventQueue) {
        if pulse == Pulse::Low {
            self.state = !self.state;
            for target in &self.outputs {
                event_queue.push(self.id, *target, self.state);
            }
        }
    }

    fn is_in_initial_state(&self) -> bool {
        self.state == Pulse::Low
    }

    fn get_outputs(&self) -> &[ModuleId] {
        &self.outputs
    }

    fn get_presses_required_for_pulse(
        &self,
        other_modules: &[Module],
        visited_stack: &mut VecDeque<String>,
    ) -> (Vec<PulseAtTime>, usize) {
        // Determine pulse times and periods of all parents
        let (mut pulses, period) = merge_periods(
            self.inputs
                .iter()
                .map(|i| {
                    let (pulses, period) = other_modules[*i]
                        .get_presses_required_for_pulse(other_modules, visited_stack);

                    (
                        pulses
                            .into_iter()
                            // We only pulse when receiving a low pulse
                            .filter(|p| p.1 == Pulse::Low)
                            .collect_vec(),
                        period,
                    )
                })
                .collect_vec(),
        );

        // If there are an odd number of elements, the second repetition will
        // have different pulse types to the first, so we need to repeat it
        if pulses.len() % 2 == 1 {
            pulses = pulses.repeat(2);
        }

        // Each pulse triggers the opposite pulse kind to the last
        let mut curr_pulse = Pulse::High;
        pulses = pulses
            .into_iter()
            .map(|mut p| {
                p.1 = curr_pulse;
                // Swap pulse kind
                curr_pulse = !curr_pulse;
                p
            })
            .collect_vec();

        (pulses, period)
    }

    fn register_input_module(&mut self, module: ModuleId) {
        self.inputs.push(module);
    }
}

#[derive(Debug, Clone)]
struct Conjunction {
    id: ModuleId,
    inputs: Vec<ModuleId>,
    outputs: Vec<ModuleId>,
    /// Each index corresponds with an input module
    memory: Vec<Option<Pulse>>,
    num_high: usize,
    total_registered: usize,
}

impl Conjunction {
    fn new(id: ModuleId, outputs: Vec<ModuleId>, num_modules: usize) -> Self {
        Conjunction {
            id,
            inputs: vec![],
            outputs,
            memory: vec![None; num_modules],
            num_high: 0,
            total_registered: 0,
        }
    }
}

impl ModuleTrait for Conjunction {
    fn receive_pulse(&mut self, input: ModuleId, pulse: Pulse, event_queue: &mut EventQueue) {
        // When a pulse is received, the conjunction module first updates its
        // memory for that input
        match pulse {
            Pulse::High => {
                if self.memory[input].unwrap() == Pulse::Low {
                    self.num_high += 1;
                    self.memory[input] = Some(Pulse::High);
                }
            }
            Pulse::Low => {
                if self.memory[input].unwrap() == Pulse::High {
                    self.num_high -= 1;
                    self.memory[input] = Some(Pulse::Low);
                }
            }
        }
        // Then, if it remembers high pulses for all inputs, it sends a low
        // pulse; otherwise, it sends a high pulse.
        let pulse_to_send = if self.num_high == self.total_registered {
            Pulse::Low
        } else {
            Pulse::High
        };

        for target in &self.outputs {
            event_queue.push(self.id, *target, pulse_to_send);
        }
    }

    fn is_in_initial_state(&self) -> bool {
        self.num_high == 0
    }

    fn register_input_module(&mut self, module: ModuleId) {
        assert!(self.memory[module].is_none());
        self.total_registered += 1;
        self.memory[module] = Some(Pulse::Low);
        self.inputs.push(module);
    }

    fn get_outputs(&self) -> &[ModuleId] {
        &self.outputs
    }

    fn get_presses_required_for_pulse(
        &self,
        other_modules: &[Module],
        visited_stack: &mut VecDeque<String>,
    ) -> (Vec<PulseAtTime>, usize) {
        // This will probably be the least efficient one.... it is kinda
        // complex
        let periods = self
            .inputs
            .iter()
            .map(|i| other_modules[*i].get_presses_required_for_pulse(other_modules, visited_stack))
            .collect_vec();

        let mut memory = vec![Pulse::Low; periods.len()];

        // Calculate the total period
        let full_period = periods
            .iter()
            .map(|(_, p)| *p)
            .reduce(num::integer::lcm)
            .unwrap();

        // Now for each input, extend its pulses to be the length of the
        // full period, then flatten them all into one sorted array
        let full_period_data = periods
            .into_iter()
            // Keep the index associated with it so that we can use it with the memory
            .enumerate()
            .flat_map(|(i, (pulses, period))| {
                pulses
                    .into_iter()
                    .map(|p| (p, i))
                    .collect_vec()
                    // Repeat it to make it last for the full period
                    .repeat(full_period / period)
            })
            .sorted()
            .collect_vec();

        // Now go through the full period data, and for each element check if
        // it would trigger a low pulse
        let mut output_period_data = vec![];

        for (pulse, i) in full_period_data {
            memory[i] = pulse.1;
            if memory.contains(&Pulse::Low) {
                // Contains a low pulse, give a high pulse
                output_period_data.push(PulseAtTime(pulse.0, Pulse::High));
            } else {
                // All high pulses, give a low pulse
                output_period_data.push(PulseAtTime(pulse.0, Pulse::Low));
            }
        }

        // Now return the data
        (output_period_data, full_period)
    }
}

#[derive(Debug, Clone)]
enum ModuleVariant {
    Broadcaster(Broadcaster),
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
}

#[derive(Debug, Clone)]
struct Module {
    name: String,
    variant: ModuleVariant,
    counts: PulseCounter,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Convert a list of output strings into a vec of module IDs
///
/// Any modules that don't exist are sent to an extra debugging one placed at
/// the end of the modules
fn load_outputs(outputs: &str, mod_names_to_ids: &HashMap<String, ModuleId>) -> Vec<ModuleId> {
    if outputs.is_empty() {
        vec![]
    } else {
        outputs
            .split(", ")
            .map(|name| {
                mod_names_to_ids
                    .get(name)
                    .copied()
                    .unwrap_or(mod_names_to_ids.len())
            })
            .collect_vec()
    }
}

impl Module {
    fn new(id: ModuleId, line: &str, mod_names_to_ids: &HashMap<String, ModuleId>) -> Module {
        let mod_type = line.chars().next().unwrap();
        let outputs = load_outputs(line.split_once(" -> ").unwrap().1, mod_names_to_ids);

        let name = extract_module_name_from_line(line);

        let variant = match mod_type {
            '%' => ModuleVariant::FlipFlop(FlipFlop::new(id, outputs)),
            '&' => {
                ModuleVariant::Conjunction(Conjunction::new(id, outputs, mod_names_to_ids.len()))
            }
            _ => ModuleVariant::Broadcaster(Broadcaster::new(id, outputs)),
        };

        Module {
            name,
            variant,
            counts: Default::default(),
        }
    }

    /// Returns the number of pulses that the module has received (low, high)
    fn get_pulse_counts(&self) -> PulseCounter {
        self.counts
    }

    fn is_broadcaster(&self) -> bool {
        matches!(self.variant, ModuleVariant::Broadcaster(_))
    }
}

impl ModuleTrait for Module {
    fn receive_pulse(&mut self, input: ModuleId, pulse: Pulse, event_queue: &mut EventQueue) {
        self.counts += pulse;
        match &mut self.variant {
            ModuleVariant::Broadcaster(v) => v.receive_pulse(input, pulse, event_queue),
            ModuleVariant::FlipFlop(v) => v.receive_pulse(input, pulse, event_queue),
            ModuleVariant::Conjunction(v) => v.receive_pulse(input, pulse, event_queue),
        }
    }

    fn is_in_initial_state(&self) -> bool {
        match &self.variant {
            ModuleVariant::Broadcaster(v) => v.is_in_initial_state(),
            ModuleVariant::FlipFlop(v) => v.is_in_initial_state(),
            ModuleVariant::Conjunction(v) => v.is_in_initial_state(),
        }
    }

    fn register_input_module(&mut self, module: ModuleId) {
        match &mut self.variant {
            ModuleVariant::Broadcaster(v) => v.register_input_module(module),
            ModuleVariant::FlipFlop(v) => v.register_input_module(module),
            ModuleVariant::Conjunction(v) => v.register_input_module(module),
        }
    }

    fn get_outputs(&self) -> &[ModuleId] {
        match &self.variant {
            ModuleVariant::Broadcaster(v) => v.get_outputs(),
            ModuleVariant::FlipFlop(v) => v.get_outputs(),
            ModuleVariant::Conjunction(v) => v.get_outputs(),
        }
    }

    fn get_presses_required_for_pulse(
        &self,
        other_modules: &[Module],
        visited_stack: &mut VecDeque<String>,
    ) -> (Vec<PulseAtTime>, usize) {
        dbg!(&self.name);

        // Track which modules we've visited to try to find out why I'm getting
        // a stack overflow :/
        if visited_stack.contains(&self.name) {
            panic!("Already visited module {:?}!!! Stack is {visited_stack:?}", self.name);
        }
        visited_stack.push_back(self.name.clone());

        let ret = match &self.variant {
            ModuleVariant::Broadcaster(v) => {
                v.get_presses_required_for_pulse(other_modules, visited_stack)
            }
            ModuleVariant::FlipFlop(v) => {
                v.get_presses_required_for_pulse(other_modules, visited_stack)
            }
            ModuleVariant::Conjunction(v) => {
                v.get_presses_required_for_pulse(other_modules, visited_stack)
            }
        };

        visited_stack.pop_back();

        ret
    }
}

fn make_mod_names_map(names: &str) -> HashMap<String, ModuleId> {
    names
        .lines()
        .enumerate()
        .map(|(i, line)| (extract_module_name_from_line(line), i))
        .collect()
}

fn extract_module_name_from_line(line: &str) -> String {
    line.replace(['%', '&'], "")
        .split_once(" -> ")
        .unwrap()
        .0
        .to_owned()
}

fn set_up_modules(input: &str) -> Vec<Module> {
    let mod_names = make_mod_names_map(input);

    let mut modules = input
        .lines()
        .enumerate()
        .map(|(id, line)| Module::new(id, line, &mod_names))
        .collect_vec();

    let outputs = modules
        .iter()
        .map(|m| m.get_outputs().iter().copied().collect_vec())
        .collect_vec();

    // Now add an extra module for debugging
    modules.push(Module::new(modules.len(), "debug -> ", &mod_names));

    for (i, outs) in outputs.into_iter().enumerate() {
        for out in outs {
            modules[out].register_input_module(i);
        }
    }

    modules
}

fn find_broadcaster_module(modules: &[Module]) -> ModuleId {
    modules
        .iter()
        .find_position(|m| m.is_broadcaster())
        .unwrap()
        .0
}

fn find_with_name(modules: &[Module], name: &str) -> ModuleId {
    modules.iter().find_position(|m| m.name == name).unwrap().0
}

#[aoc(day20, part1)]
pub fn part_1(input: &str) -> usize {
    let mut modules = set_up_modules(input);

    let broadcaster_id = find_broadcaster_module(&modules);

    let mut event_queue = EventQueue::default();

    let mut push_count = 0;

    while push_count == 0 || !modules.iter().all(|m| m.is_in_initial_state()) {
        event_queue.push(broadcaster_id, broadcaster_id, Pulse::Low);
        event_queue.drain(&mut modules);
        push_count += 1;
        if push_count == 1000 {
            break;
        }
    }

    let pulses_per_cycle: PulseCounter = modules.iter().map(|m| m.get_pulse_counts()).sum();

    let num_cycles = 1000 / push_count;
    let remaining_pushes = 1000 - num_cycles * push_count;

    for _ in 0..remaining_pushes {
        event_queue.push(broadcaster_id, broadcaster_id, Pulse::Low);
        event_queue.drain(&mut modules);
    }

    usize::from(
        modules
            .iter()
            .map(|m| m.get_pulse_counts())
            .sum::<PulseCounter>()
            + pulses_per_cycle * (num_cycles - 1),
    )
}

#[aoc(day20, part2)]
pub fn part_2(input: &str) -> usize {
    // unsafe {
    //     backtrace_on_stack_overflow::enable();
    // }
    let modules = set_up_modules(input);
    let rx_id = find_with_name(&modules, "debug");

    modules[rx_id]
        .get_presses_required_for_pulse(&modules, &mut VecDeque::default())
        // Discard the full duration, since it will happen somewhere in the
        // period
        .0
        .into_iter()
        // Find the first low pulse
        .find(|p| p.1 == Pulse::Low)
        .unwrap()
        // Grab the button press that it happens at
        // and add 1, since we started time at zero
        .0
        + 1
}

#[cfg(test)]
mod test {
    use super::{part_1, part_2};

    #[test]
    fn test_part_1_simple() {
        assert_eq!(
            part_1(
                "broadcaster -> a, b, c\n\
                %a -> b\n\
                %b -> c\n\
                %c -> inv\n\
                &inv -> a"
            ),
            32000000,
        )
    }

    #[test]
    fn test_part_1_advanced() {
        assert_eq!(
            part_1(
                "broadcaster -> a\n\
                %a -> inv, con\n\
                &inv -> b\n\
                %b -> con\n\
                &con -> output"
            ),
            11687500,
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "broadcaster -> a\n\
                %a -> inv, con\n\
                &inv -> b\n\
                %b -> con\n\
                &con -> output"
            ),
            1
        )
    }
}
