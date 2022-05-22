#![feature(box_syntax)]

use itertools::Itertools;
use std::ops::BitXor;

struct Circuit {
    num_inputs: usize,
    num_outputs: usize,
    gates: Vec<Box<dyn Gate>>,
}

fn main() {
    let gates: Vec<Box<dyn Gate>> =
        vec![box Not {}, box Not {}, box BitSwitch {}, box BitSwitch {}];
    for combo in gates.iter().permutations(gates.len()) {}
}

trait Gate {
    fn input_count(&self) -> usize;
    fn output_count(&self) -> usize;
    fn output(&self, inputs: &[bool]) -> Vec<bool>;
}

struct Not {}

impl Gate for Not {
    fn input_count(&self) -> usize {
        1
    }

    fn output_count(&self) -> usize {
        1
    }

    fn output(&self, inputs: &[bool]) -> Vec<bool> {
        vec![!inputs[0]]
    }
}

struct BitSwitch {}

impl Gate for BitSwitch {
    fn input_count(&self) -> usize {
        2
    }

    fn output_count(&self) -> usize {
        1
    }

    fn output(&self, inputs: &[bool]) -> Vec<bool> {
        vec![inputs[0] && inputs[1]]
    }
}

fn desired_truth_table(inputs: &[bool]) -> bool {
    inputs.iter().fold(false, BitXor::bitxor)
}
