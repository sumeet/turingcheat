#![feature(box_syntax)]

use dyn_clonable::clonable;
use std::collections::HashMap;
use std::iter::once;
use std::ops::BitXor;

fn main() {
    const NUM_INPUTS: usize = 2;
    const NUM_OUTPUTS: usize = 1;
    let gates: Vec<Box<dyn Gate>> =
        vec![box Not {}, box Not {}, box BitSwitch {}, box BitSwitch {}];
    #[allow(unused)]
    let all_connection_indices = generate_all_connection_indices(NUM_INPUTS, NUM_OUTPUTS, &gates);

    let inputs = gen_inputs::<NUM_INPUTS>();
    let circuit = Circuit {
        num_outputs: NUM_OUTPUTS,
        gates: gates.clone(),
        connections: HashMap::new(),
    };
    let output = circuit.run(&inputs[0]);
    dbg!(output);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
enum ConnectionIndex {
    Input(usize),
    Output(usize),
    GateInput { gate_index: usize, io_index: usize },
    GateOutput { gate_index: usize, io_index: usize },
}

struct Circuit {
    num_outputs: usize,
    gates: Vec<Box<dyn Gate>>,
    connections: HashMap<ConnectionIndex, ConnectionIndex>,
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
            let dest = self
                .connections
                .get(&ConnectionIndex::Input(i))
                .ok_or_else(|| format!("No connection for input {}", i))
                .unwrap();
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
                ConnectionIndex::Input(_) | ConnectionIndex::GateOutput { .. } => unreachable!(),
            }
        }

        while let Some((gate_index, inputs)) =
            set_inputs_by_gate
                .iter()
                .enumerate()
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
            let triggered_outputs = self.gates.get(gate_index).unwrap().trigger(&inputs);
            for (i, output) in triggered_outputs.iter().enumerate() {
                let dest = self
                    .connections
                    .get(&ConnectionIndex::GateOutput {
                        gate_index,
                        io_index: i,
                    })
                    .unwrap();
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
        outputs
    }
}

fn generate_all_connection_indices(
    num_inputs: usize,
    num_outputs: usize,
    gates: &[Box<dyn Gate>],
) -> Vec<ConnectionIndex> {
    let mut connection_indices = Vec::new();
    for i in 0..num_inputs {
        connection_indices.push(ConnectionIndex::Input(i));
    }
    for i in 0..num_outputs {
        connection_indices.push(ConnectionIndex::Output(i));
    }
    for (gate_index, gate) in gates.iter().enumerate() {
        for io_index in 0..gate.num_inputs() {
            connection_indices.push(ConnectionIndex::GateInput {
                gate_index,
                io_index,
            });
        }
        for io_index in 0..gate.num_outputs() {
            connection_indices.push(ConnectionIndex::GateOutput {
                gate_index,
                io_index,
            });
        }
    }
    connection_indices
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
        vec![inputs[0] && inputs[1]]
    }
}

#[allow(unused)]
fn desired_truth_table(inputs: &[bool]) -> bool {
    inputs.iter().fold(false, BitXor::bitxor)
}
