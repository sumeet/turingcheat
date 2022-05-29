#![feature(box_syntax)]

use dyn_clonable::clonable;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::iter::{empty, once};
use std::ops::BitXor;

fn main() {
    let gates: Vec<Box<dyn Gate>> =
        vec![box Not {}, box Not {}, box BitSwitch {}, box BitSwitch {}];
    const NUM_INPUTS: usize = 2;
    const NUM_OUTPUTS: usize = 1;
    let all_connection_indices = generate_all_connection_indices(NUM_INPUTS, NUM_OUTPUTS, &gates);
    let all_connection_sets = gen_all_connection_sets(&all_connection_indices).collect_vec();
    for connections in all_connection_sets {
        let circuit = Circuit {
            num_outputs: NUM_OUTPUTS,
            gates: gates.clone(),
            connections,
        };

        if test_circuit::<NUM_INPUTS>(&circuit) {
            dbg!(circuit.connections);
            return;
        }
    }
    println!("got to the end :(")
}

// TODO: xor_desired_truth_table can be an argument
fn test_circuit<const N: usize>(circuit: &Circuit) -> bool {
    let inputs = gen_inputs::<N>();
    let ct = inputs
        .iter()
        .filter(|inputs| {
            let expected = xor_desired_truth_table(inputs.as_slice())
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>();
            let got = circuit.run(inputs.as_slice());
            expected == got
        })
        .count();
    ct == inputs.len()
}

fn xor_desired_truth_table(inputs: &[bool]) -> Vec<bool> {
    vec![inputs.iter().fold(false, BitXor::bitxor)]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
enum ConnectionIndex {
    Input(usize),
    Output(usize),
    GateInput { gate_index: usize, io_index: usize },
    GateOutput { gate_index: usize, io_index: usize },
}

type Connections = HashMap<ConnectionIndex, Vec<ConnectionIndex>>;

struct Circuit {
    num_outputs: usize,
    gates: Vec<Box<dyn Gate>>,
    connections: Connections,
}

impl Circuit {
    fn run(&self, inputs: &[bool]) -> Vec<Option<bool>> {
        let mut outputs = vec![None; self.num_outputs];

        let mut set_inputs_by_gate = self
            .gates
            .iter()
            .map(|gate| once(None).cycle().take(gate.num_inputs()).collect())
            .collect::<Vec<Vec<Option<bool>>>>();

        for (i, input) in inputs.iter().enumerate() {
            let dests = self
                .connections
                .get(&ConnectionIndex::Input(i))
                .ok_or_else(|| format!("No connection for input {}", i))
                .unwrap();
            for dest in dests {
                match dest {
                    ConnectionIndex::GateInput {
                        gate_index,
                        io_index,
                    } => {
                        set_inputs_by_gate[*gate_index][*io_index] = Some(*input);
                    }
                    ConnectionIndex::Output(output_index) => {
                        outputs[*output_index] = Some(*input);
                    }
                    ConnectionIndex::Input(_) | ConnectionIndex::GateOutput { .. } => {
                        unreachable!()
                    }
                }
            }
        }

        let mut used_input_gates = HashSet::new();
        while let Some((gate_index, inputs)) = set_inputs_by_gate
            .iter()
            .enumerate()
            .filter(|(gate_index, _)| !used_input_gates.contains(gate_index))
            .find_map(|(i, inputs)| {
                let is_all_inputs_set = inputs.iter().all(|input| input.is_some());
                if !is_all_inputs_set {
                    return None;
                }
                let inputs = inputs
                    .iter()
                    .map(|input| input.unwrap())
                    .collect::<Vec<_>>();
                Some((i, inputs))
            })
        {
            used_input_gates.insert(gate_index);

            let gate = self.gates.get(gate_index).unwrap();
            if !gate.is_on(&inputs) {
                continue;
            }
            let triggered_outputs = gate.trigger(&inputs);
            for (i, output) in triggered_outputs.iter().enumerate() {
                let dests = self
                    .connections
                    .get(&ConnectionIndex::GateOutput {
                        gate_index,
                        io_index: i,
                    })
                    .ok_or_else(|| format!("No connection for gate input {} {}", gate_index, i))
                    .unwrap();
                for dest in dests {
                    match dest {
                        ConnectionIndex::GateInput {
                            gate_index,
                            io_index,
                        } => {
                            set_inputs_by_gate[*gate_index][*io_index] = Some(*output);
                        }
                        ConnectionIndex::Output(output_index) => {
                            outputs[*output_index] = Some(*output);
                        }
                        ConnectionIndex::Input(_) => {
                            unreachable!()
                        }
                        ConnectionIndex::GateOutput { .. } => unreachable!(),
                    }
                }
            }
        }
        outputs
    }
}

#[derive(Debug)]
struct Connectables {
    sources: HashSet<ConnectionIndex>,
    switch_sources: HashSet<ConnectionIndex>,
    dests: HashSet<ConnectionIndex>,
}

impl Connectables {
    fn new(
        sources: HashSet<ConnectionIndex>,
        switch_sources: HashSet<ConnectionIndex>,
        dests: HashSet<ConnectionIndex>,
    ) -> Self {
        Self {
            sources,
            switch_sources,
            dests,
        }
    }
}

fn generate_all_connection_indices(
    num_inputs: usize,
    num_outputs: usize,
    gates: &[Box<dyn Gate>],
) -> Connectables {
    let mut sources = HashSet::new();
    let mut switch_sources = HashSet::new();
    let mut dests = HashSet::new();
    for i in 0..num_inputs {
        sources.insert(ConnectionIndex::Input(i));
    }
    for i in 0..num_outputs {
        dests.insert(ConnectionIndex::Output(i));
    }
    for (gate_index, gate) in gates.iter().enumerate() {
        for io_index in 0..gate.num_inputs() {
            dests.insert(ConnectionIndex::GateInput {
                gate_index,
                io_index,
            });
        }
        for io_index in 0..gate.num_outputs() {
            let output = ConnectionIndex::GateOutput {
                gate_index,
                io_index,
            };
            if gate.is_switch() {
                switch_sources.insert(output);
            } else {
                sources.insert(output);
            }
        }
    }
    Connectables::new(sources, switch_sources, dests)
}

fn gen_all_connection_sets(connectables: &Connectables) -> impl Iterator<Item = Connections> + '_ {
    let connections = Connections::new();
    gen_remaining_connection_sets(connectables, connections)
}

fn gen_remaining_connection_sets<'a>(
    connectables: &'a Connectables,
    connections: Connections,
) -> Box<dyn Iterator<Item = Connections> + 'a> {
    let all_used_dests = connections
        .values()
        .flatten()
        .copied()
        .collect::<HashSet<_>>();

    let next_unplugged_dest = connectables.dests.difference(&all_used_dests).next();
    if next_unplugged_dest.is_none() {
        // check to see if all inputs are used?
        return if connectables.sources.len() != connections.len() {
            box empty()
        } else {
            box once(connections)
        };
    }

    let next_unplugged_dest = next_unplugged_dest.copied().unwrap();
    box connectables.sources.iter().flat_map(move |source| {
        let mut connections = connections.clone();
        connections
            .entry(*source)
            .or_insert_with(|| vec![])
            .push(next_unplugged_dest);
        if contains_infinite_loop(&connections) {
            box empty()
        } else {
            gen_remaining_connection_sets(connectables, connections)
        }
    })
}

fn contains_infinite_loop_rec(
    connections: &Connections,
    from: ConnectionIndex,
    visited_gates: &mut HashSet<usize>,
) -> bool {
    let outs = connections.get(&from).unwrap();
    for out in outs {
        match out {
            ConnectionIndex::Input(_) | ConnectionIndex::GateOutput { .. } => {
                panic!("unexpected, input as destination")
            }
            ConnectionIndex::Output(_) => {
                // end of the road, no infinite loop
                continue;
            }
            ConnectionIndex::GateInput {
                gate_index: this_output_index,
                ..
            } => {
                if visited_gates.contains(this_output_index) {
                    return true;
                }
                visited_gates.insert(*this_output_index);

                let nexts = connections.keys().filter(|conn_index| match conn_index {
                    ConnectionIndex::Input(_) => false,
                    ConnectionIndex::Output(_) | ConnectionIndex::GateInput { .. } => {
                        unreachable!()
                    }
                    ConnectionIndex::GateOutput {
                        gate_index: next_index,
                        ..
                    } => this_output_index == next_index,
                });
                for next in nexts {
                    if contains_infinite_loop_rec(connections, *next, &mut visited_gates.clone()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn contains_infinite_loop(connections: &Connections) -> bool {
    let mut input_gates = connections
        .keys()
        .filter(|conn_index| matches!(conn_index, ConnectionIndex::Input(_)));
    input_gates.any(|conn_index| {
        let mut visited = HashSet::new();
        match conn_index {
            ConnectionIndex::Input(index) => {
                visited.insert(*index);
            }
            _ => unreachable!(),
        }
        contains_infinite_loop_rec(connections, *conn_index, &mut visited)
    })
}

fn gen_inputs<const N: usize>() -> Vec<[bool; N]> {
    (0..1 << N)
        .map(|n| {
            let mut inputs = [false; N];
            for i in 0..N {
                inputs[i] = (n & (1 << i)) != 0;
            }
            inputs
        })
        .collect()
}

#[clonable]
trait Gate: Clone {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn trigger(&self, inputs: &[bool]) -> Vec<bool>;
    fn is_on(&self, _inputs: &[bool]) -> bool {
        true
    }
    fn is_switch(&self) -> bool {
        false
    }
}

#[derive(Clone)]
struct Not {}

impl Gate for Not {
    fn num_inputs(&self) -> usize {
        1
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn trigger(&self, inputs: &[bool]) -> Vec<bool> {
        vec![!inputs[0]]
    }
}

#[derive(Clone)]
struct BitSwitch {}

impl Gate for BitSwitch {
    fn num_inputs(&self) -> usize {
        2
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn trigger(&self, inputs: &[bool]) -> Vec<bool> {
        vec![inputs[1]]
    }
    fn is_on(&self, inputs: &[bool]) -> bool {
        inputs[0]
    }
    fn is_switch(&self) -> bool {
        true
    }
}

#[test]
fn test_contains_infinite_loop() {
    // input directly connected to output
    let mut connections = Connections::new();
    connections.insert(ConnectionIndex::Input(0), vec![ConnectionIndex::Output(0)]);
    assert!(!contains_infinite_loop(&connections));

    // gate going to itself
    let mut connections = Connections::new();
    connections.insert(
        ConnectionIndex::Input(0),
        vec![ConnectionIndex::GateInput {
            gate_index: 0,
            io_index: 0,
        }],
    );
    connections.insert(
        ConnectionIndex::GateOutput {
            gate_index: 0,
            io_index: 0,
        },
        vec![ConnectionIndex::GateInput {
            gate_index: 0,
            io_index: 0,
        }],
    );
    assert!(contains_infinite_loop(&connections));

    // gate going through itself through another gate
    let mut connections = Connections::new();
    connections.insert(
        ConnectionIndex::Input(0),
        vec![ConnectionIndex::GateInput {
            gate_index: 0,
            io_index: 0,
        }],
    );
    connections.insert(
        ConnectionIndex::GateOutput {
            gate_index: 0,
            io_index: 0,
        },
        vec![ConnectionIndex::GateInput {
            gate_index: 1,
            io_index: 0,
        }],
    );
    connections.insert(
        ConnectionIndex::GateOutput {
            gate_index: 1,
            io_index: 0,
        },
        vec![ConnectionIndex::GateInput {
            gate_index: 0,
            io_index: 0,
        }],
    );
    assert!(contains_infinite_loop(&connections));
}
