extern crate petgraph;
extern crate rand;
extern crate rayon;

use std::sync::{RwLock, Arc};

use petgraph::prelude::*;
use petgraph::graph::EdgeReference;
use petgraph::algo::{DfsSpace, has_path_connecting, kosaraju_scc};
use petgraph::dot::Dot;

use rand::{SeedableRng, distributions::{Sample, Range}};

use rayon::prelude::*;

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

    let mut max_weight = RwLock::new((0, 0.0));
    let mut phisical = Graph::new();
    for i in 0..N {
        phisical.add_node(i);
    }

    (0usize..N).into_par_iter().for_each(|start| {
        let mut tot_weight = 0.0;
        let mut connected = vec![false; N];
        let mut unconnected = N-1;
        connected[start] = true;
        let mut a = NodeIndex::new(start);
        while unconnected > 0 {
            let e:EdgeReference<_> = logic.edges_directed(a, Direction::Outgoing)
                .filter(|e| !connected[e.target().index()])
                .max_by(|e1, e2| {
                    match e2.weight().partial_cmp(e1.weight()) {
                        Some(o) => o,
                        None => std::cmp::Ordering::Equal
                    }
                }).unwrap();
            unconnected -= 1;
            connected[e.target().index()] = true;
            a = e.target();
            tot_weight += e.weight();
        }
        let e = logic.find_edge(a, NodeIndex::new(start)).unwrap();
        tot_weight += logic.edge_weight(e).unwrap();
        if max_weight.read().unwrap().1 < tot_weight {
            *max_weight.write().unwrap() = (start, tot_weight);
        }
    });

    let start = max_weight.into_inner().unwrap().0;
    let mut connected = vec![false; N];
    let mut unconnected = N-1;
    connected[start] = true;
    let mut a = NodeIndex::new(start);
    while unconnected > 0 {
        let e:EdgeReference<_> = logic.edges_directed(a, Direction::Outgoing)
            .filter(|e| !connected[e.target().index()])
            .max_by(|e1, e2| {
                match e2.weight().partial_cmp(e1.weight()) {
                    Some(o) => o,
                    None => std::cmp::Ordering::Equal
                }
            }).unwrap();
        phisical.add_edge(a, e.target(), e.weight());
        unconnected -= 1;
        connected[e.target().index()] = true;
        a = e.target();
    }
    let e = logic.find_edge(a, NodeIndex::new(start)).unwrap();
    phisical.add_edge(a, NodeIndex::new(start), logic.edge_weight(e).unwrap());
    
    
    let dot = Dot::new(&phisical);
    println!("{:?}", dot);
}

