#![feature(box_syntax)]

use dyn_clonable::clonable;
use itertools::Itertools;
use std::ops::BitXor;

fn main() {
    const NUM_INPUTS: usize = 2;
    const NUM_OUTPUTS: usize = 1;
    let gates: Vec<Box<dyn Gate>> =
        vec![box Not {}, box Not {}, box BitSwitch {}, box BitSwitch {}];
    let all_connection_indices = generate_all_connection_indices(NUM_INPUTS, NUM_OUTPUTS, &gates);

    let inputs = gen_inputs::<NUM_INPUTS>();
    let circuit = Circuit {
        num_inputs: NUM_INPUTS,
        num_outputs: NUM_OUTPUTS,
        gates: gates.clone(),
        connections: Vec::new(),
    };
    let output = circuit.run(&inputs[0]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ConnectionIndex {
    Input(usize),
    Output(usize),
    GateInput { gate_index: usize, io_index: usize },
    GateOutput { gate_index: usize, io_index: usize },
}

struct Connection {
    input: ConnectionIndex,
    output: ConnectionIndex,
}

struct Circuit {
    num_inputs: usize,
    num_outputs: usize,
    gates: Vec<Box<dyn Gate>>,
    connections: Vec<Connection>,
}

impl Circuit {
    pub(crate) fn run(&self, p0: &[bool]) -> _ {
        todo!()
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
    fn trigger(&mut self, inputs: &[bool]) -> Vec<bool>;
}

#[derive(Clone)]
struct Output {
    received: Option<Vec<bool>>,
}

impl Output {
    fn new() -> Self {
        Self { received: None }
    }
}

impl Gate for Output {
    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn trigger(&mut self, inputs: &[bool]) -> Vec<bool> {
        self.received = Some(inputs.to_vec());
        vec![]
    }
}

impl Gate for Input {
    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn trigger(&mut self, _: &[bool]) -> Vec<bool> {
        vec![self.signal]
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

    fn trigger(&mut self, inputs: &[bool]) -> Vec<bool> {
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

    fn trigger(&mut self, inputs: &[bool]) -> Vec<bool> {
        vec![inputs[0] && inputs[1]]
    }
}

fn desired_truth_table(inputs: &[bool]) -> bool {
    inputs.iter().fold(false, BitXor::bitxor)
}
