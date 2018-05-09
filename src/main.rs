extern crate petgraph;
extern crate rand;

use petgraph::prelude::*;
use petgraph::dot::{Dot, Config};
use rand::{SeedableRng, distributions::{Sample, Range}};

const N: usize = 20;
const DELTA: usize = 1;

fn main() {
    let mut rng = rand::XorShiftRng::from_seed([1, 2, 3, 4]);
    let mut rng2 = rand::XorShiftRng::from_seed([4, 2, 3, 0]);
    let mut traffic_values = Range::new(0.0, 1.0);
    let mut range = Range::new(0usize, N);
    let mut logic = Graph::new();

    for _ in 0..N {
        logic.add_node(1);
    }

    for i in 0usize..N {
        for _ in 0usize..DELTA {
            let mut to = range.sample(&mut rng2);
            while logic.edges_directed(NodeIndex::new(to), Direction::Incoming).count() >= DELTA || to == i {
                to = range.sample(&mut rng2);
            }
            logic.add_edge(NodeIndex::new(i), NodeIndex::new(to), traffic_values.sample(&mut rng));
        }
    }

    let dot = Dot::with_config(&logic, &[]);
    println!("{:?}", dot);
}
