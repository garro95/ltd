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
    let mut connected = vec![false; N];
    let mut unconnected = N;
    let mut indegree = vec![0;N];
    let mut outdegree = vec![0, N];
    let mut phisical = Graph::new();
    for i in 0..N {
        phisical.add_node(i);
    }
    // Take the first edge
    let e = &sorted_edges[0];
    let a = e.source();
    let b = e.source();
    phisical.add_edge(a, b, e.weight);
    connected[a.index()] = true;
    outdegree[a.index()] +=1;
    connected[b.index()] = true;
    indegree[b.index()] += 1;
    // in this first iteration, take the most heavy edges that strongly connects the nodes
    for e in sorted_edges.iter().skip(1) {
        let a = e.source();
        let b = e.target();
        // check if the target node is not yet connected
        if !connected[b.index()] {
            // check that none of the nodes has yet reached its maximum indegree/outdegree
            if outdegree[a.index()] < DELTA && indegree[b.index()] < DELTA {
                phisical.add_edge(a, b, e.weight);
                outdegree[a.index()] +=1;
                connected[b.index()] = true;
                unconnected -= 1;
                indegree[b.index()] += 1;
            }
        }
        if unconnected == 0 {break}
    }
    // now that all the nodes can reach each other, 

    let dot = Dot::new(&phisical);
    println!("{:?}", dot);
}

