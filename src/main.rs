extern crate petgraph;
extern crate rand;

use petgraph::prelude::*;
use petgraph::dot::Dot;
use rand::{SeedableRng, distributions::{Sample, Range}};

const N: usize = 20;
const DELTA: usize = 1;

fn main() {
    let mut rng = rand::XorShiftRng::from_seed([1, 2, 3, 4]);
    let mut traffic_values = Range::new(0.0, 1.0);
    let mut logic = Graph::new();

    for _ in 0..N {
        logic.add_node(1);
    }

    for from in 0usize..N {
        for to in (0usize..N).filter(|&a| a != from) {
            logic.add_edge(NodeIndex::new(from), NodeIndex::new(to), traffic_values.sample(&mut rng));
        }
    }

    let dot = Dot::new(&logic);
    println!("{:?}", dot);
}
