extern crate petgraph;
extern crate rand;

use petgraph::prelude::*;
use petgraph::algo::{DfsSpace, has_path_connecting, kosaraju_scc};
use petgraph::dot::Dot;
use rand::{SeedableRng, distributions::{Sample, Range}};

const N: usize = 20;
const DELTA: usize = 1;

fn main() {
    let mut rng = rand::XorShiftRng::from_seed([1, 2, 3, 4]);
    let mut traffic_values = Range::new(0.0, 1.0);

    // Create a logical topology that describes the traffic from one node to each other.
    let mut logic = Graph::new();

    for _ in 0..N {
        logic.add_node(1);
    }

    for from in 0usize..N {
        for to in (0usize..N).filter(|&a| a != from) {
            logic.add_edge(NodeIndex::new(from), NodeIndex::new(to), traffic_values.sample(&mut rng));
        }
    }

    /* let dot = Dot::new(&logic);
    println!("{:?}", dot); */
    // Algorithm:
    // 1. Order the arcs by decreasing weight
    // 2. If the arc connects two nodes not connected, take it.
    // 3. When all the graph is strongly connected, if there are still link available that can be used respecting DELTA, take them in order

    let mut sorted_edges = Vec::from(logic.raw_edges());
    sorted_edges.sort_unstable_by(|e1, e2| {
        match e2.weight.partial_cmp(&e1.weight) {
            Some(o) => o,
            None => std::cmp::Ordering::Equal
        }
    });
    //let mut connected = vec![false; N];
    //let mut unconnected = N;
    let mut indegree = vec![0;N];
    let mut outdegree = vec![0;N];
    let mut phisical = Graph::new();
    for i in 0..N {
        phisical.add_node(i);
    }
    // Try to take the heaviest Hamilton Cycle
    let mut workspace = DfsSpace::new(&phisical);
    for e in &sorted_edges {
        let a = e.source();
        let b = e.target();
        let w = e.weight;
        if outdegree[a.index()] < DELTA && indegree[b.index()] < DELTA && 
            !has_path_connecting(&phisical, a, b, Some(&mut workspace)) {
                phisical.add_edge(a, b, w);
                outdegree[a.index()] += 1;
                indegree[b.index()] += 1;
        }
    }

    // If there are more than one sccs, try to connect them
    let sccs = kosaraju_scc(&phisical);
    if sccs.len() > 1 {
        
    }
    

    let dot = Dot::new(&phisical);
    println!("{:?}", dot);
}

