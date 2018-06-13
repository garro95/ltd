/*
 *  Copyright © 2018 Gianmarco Garrisi
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
extern crate petgraph;
extern crate rand;
extern crate rayon;
#[macro_use] extern crate quicli;

use std::sync::RwLock;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ffi::OsString;
use std::io::Write;
use std::cmp::Ordering;

use petgraph::prelude::*;
use petgraph::graph::EdgeReference;
use petgraph::dot::Dot;
use petgraph::algo::astar;
use petgraph::visit::{
    EdgeRef,
};

use rand::{SeedableRng, XorShiftRng, distributions::{Sample, Range}};

use quicli::prelude::*;

fn parse_seed(seed: &str) -> [u32;4] {
    let mut seed:u128 = seed.parse().expect("Unable to parse random seed");
    let mut parsed = [0;4];
    for i in 1..=4 {
        parsed[4-i] = (seed & std::u32::MAX as u128) as u32;
        seed = seed >> 32;
    }
    parsed
}

/// Heuristically compute a good physical topology to deliver the
/// traffic generated by a randomly generated, completely connected
/// logical network
#[derive(Debug, StructOpt)]
struct Interface {
    /// Number of nodes in the network
    #[structopt(long="nodes", short="n")]
    n: usize,
    /// Number of lasers and receivers. This determins the number of
    /// physical links that can enter and exit a node
    #[structopt(long="delta", short="D")]
    delta: usize,
    /// Random seed. The traffic is genearated using a seeded random number generator,
    /// in order to be consistent between two run. You can change it to change the result
    /// of the computation. Can be an integer ranging from 1 up to 340,282,366,920,938,463,463,374,607,431,768,211,455
    #[structopt(long="seed", short="s", default_value="1234", parse(from_str="parse_seed"))]
    seed: [u32;4],
    /// If specified, the final computed graph will be output on the specified file using graphviz.
    /// The type of the output file will be desumed from the extension. If the extension is not
    /// specified, will default to dot
    #[structopt(long="output-file", short="o", parse(from_os_str))]
    output_file: Option<PathBuf>,
    /// Specify if it is possible to split the traffic on one node in order to send it
    /// in parallel on more lightpaths.
    #[structopt(long="split", short="S")]
    splittable: bool,
    /// Specify if a manhattan topology should be used and, in that case, the length of a row
    #[structopt(long="manhattan", short="m")]
    manhattan: Option<usize>
}

main!(|args:Interface| {
    let mut rng = XorShiftRng::from_seed(args.seed);
    let logical = create_logical_topology(args.n, &mut rng);
    // Heuristically find a heavy Hamilton cycle (like a multi start)
    let mut phisical = if let Some(r) = args.manhattan {
        create_manhattan_physical_topology(logical.node_count(), r)
    } else {
        create_opportunistic_physical_topology(&logical, args.delta)
    };

    // Assign all the traffic to the existing paths.
    let edges = Vec::from(phisical.raw_edges());
    for (i, e) in phisical.edge_weights_mut().enumerate() {
        let (from, to) = (edges[i].source(), edges[i].target());
        let idx = logical.find_edge(from, to).unwrap();
        let weight = logical.edge_weight(idx).unwrap();
        *e = *weight;
    }
    let mut sorted_edges = Vec::from(logical.raw_edges());
    sorted_edges.par_sort_unstable_by(|e1, e2| {
        match e2.weight.partial_cmp(&e1.weight) {
            Some(o) => o,
            None => std::cmp::Ordering::Equal
        }
    });
    for e in sorted_edges {
        if !phisical.contains_edge(e.source(), e.target()) {
            let path = astar(&phisical, e.source(), |t| t == e.target(), |e| *e.weight(), |_| 0.0).unwrap().1;
            let mut a = path[0];
            for b in path.into_iter().skip(1) {
                let te = phisical.find_edge(a, b).unwrap();
                *(phisical.edge_weight_mut(te).unwrap()) += e.weight;
                a = b;
            }
        }        
    }

    if args.splittable {
        for _ in 0..args.n/3 {
            let mut p = phisical.clone();
            let e = p.raw_edges().par_iter()
                .max_by(|e1, e2| {
                    match e1.weight.partial_cmp(&e2.weight) {
                        Some(o) => o,
                        None => std::cmp::Ordering::Equal
                    }
                }).unwrap().clone();
            let etr = {p.find_edge(e.source(), e.target())}.unwrap();
            p.remove_edge(etr).unwrap();
            let path = astar(&p, e.source(), |t| t==e.target(), |e| *e.weight(), |_| 0.0).unwrap().1;
            let mut a = path[0];
            let m = path.iter().zip(path.iter().skip(1))
                .map(|(a, b)| phisical.find_edge(*a, *b).unwrap())
                .max_by(|e1, e2| {
                    match phisical.edge_weight(*e1).unwrap().partial_cmp(phisical.edge_weight(*e2).unwrap()) {
                        Some(o) => o,
                        None => Ordering::Equal
                    }
                });
            let m = if let Some(m) = m {
                m
            } else {
                println!("{:?}", path);
                continue
            };
            let val = phisical.edge_weight(m).unwrap().clone();
            let new_weight = (e.weight + val)/2.0;
            let overflow = e.weight - new_weight;
            for b in path.into_iter().skip(1) {
                let te = phisical.find_edge(a, b).unwrap();
                *(phisical.edge_weight_mut(te).unwrap()) += overflow;
                a = b;
            }
            let te = phisical.find_edge(e.source(), e.target()).unwrap();
            *(phisical.edge_weight_mut(te).unwrap()) = new_weight;
        }
    }
    println!("{}", phisical.edge_weights_mut().max_by(|a, b| {
        match a.partial_cmp(&b) {
            Some(o) => o,
            None => std::cmp::Ordering::Equal
        }
    }).unwrap());
    if let Some(output_file) = args.output_file {
        output_result(&phisical, output_file);
    }
    assert_eq!(
        (0..args.n).into_par_iter()
            .map(|i| NodeIndex::new(i))
            .map(|i|
                 phisical.edges_directed(i, Direction::Outgoing).map(|er| er.weight()).sum::<f64>()
                 + logical.edges_directed(i, Direction::Incoming).map(|er| er.weight()).sum::<f64>()
                 - phisical.edges_directed(i, Direction::Incoming).map(|er| er.weight()).sum::<f64>()
                 - logical.edges_directed(i, Direction::Outgoing).map(|er| er.weight()).sum::<f64>())
            .filter(|d| *d>0.01)
            .map(|d| {println!("{}", d); d})
            .count(), 0, "Unfeasible solution found: Not all traffic has been delivered or more then expected has been");


});

fn create_logical_topology(nodes:usize, rng: &mut XorShiftRng) -> Graph<usize, f64>{
    let mut traffic_values = Range::new(0.0, 1.0);

    // Create a logical topology that describes the traffic from one node to each other.
    let mut logic = Graph::new();

    for i in 0..nodes {
        logic.add_node(i);
    }

    for from in 0usize..nodes {
        for to in (0usize..nodes).filter(|&a| a != from) {
            logic.add_edge(NodeIndex::new(from), NodeIndex::new(to), traffic_values.sample(rng));
        }
    }
    logic
}

fn find_start(logic: &Graph<usize, f64>) -> usize {
    let max_weight = RwLock::new((0, 0.0));
    let n = logic.node_count();

    (0usize..n).into_par_iter().for_each(|start| {
        let mut tot_weight = 0.0;
        let mut connected = vec![false; n];
        let mut unconnected = n-1;
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

    max_weight.into_inner().unwrap().0
}

fn create_opportunistic_physical_topology(logical: &Graph<usize, f64>, delta: usize)
                                          -> Graph<usize, f64>{
    let start = find_start(&logical);
    let n = logical.node_count();
    let (mut indegree, mut outdegree) = (vec![0;n], vec![0;n]);
    // Create a graph containing the found cycle
    let mut phisical: Graph<usize, f64> = Graph::new();
    for i in 0..n {
        phisical.add_node(i);
    }

    let mut connected = vec![false; n];
    let mut unconnected = n-1;
    connected[start] = true;
    let mut a = NodeIndex::new(start);
    while unconnected > 0 {
        let e:EdgeReference<_> = logical.edges_directed(a, Direction::Outgoing)
            .filter(|e| !connected[e.target().index()])
            .max_by(|e1, e2| {
                match e2.weight().partial_cmp(e1.weight()) {
                    Some(o) => o,
                    None => std::cmp::Ordering::Equal
                }
            }).unwrap();
        phisical.add_edge(a, e.target(), 0.0);
        unconnected -= 1;
        connected[e.target().index()] = true;
        outdegree[a.index()] += 1;
        indegree[e.target().index()] += 1;
        a = e.target();
    }
    phisical.add_edge(a, NodeIndex::new(start), 0.0);
    outdegree[a.index()] += 1;
    indegree[start] += 1;

    // If there is still space, try to add more arcs, starting from the heaviest
    let mut sorted_edges = Vec::from(logical.raw_edges());
    sorted_edges.par_sort_unstable_by(|e1, e2| {
        match e2.weight.partial_cmp(&e1.weight) {
            Some(o) => o,
            None => std::cmp::Ordering::Equal
        }
    });
    if delta > 1 {
        for e in &sorted_edges {
            if outdegree[e.source().index()] < delta && indegree[e.target().index()] < delta
                && !phisical.contains_edge(e.source(), e.target()){
                    phisical.add_edge(e.source(), e.target(), 0.0);
                    outdegree[e.source().index()] += 1;
                    indegree[e.target().index()] += 1;
                }
        }
    }
    phisical
}

fn create_manhattan_physical_topology(n:usize, r:usize) -> Graph<usize, f64>{
    if n%r != 0 {
        panic!("Cannot build a manhattan topology with the given dimentions");
    }
    let c = n/r;
    let left = |i| {
        if i % r == 0 {
            i + r - 1
        } else {
            i - 1
        }
    };
    let right = |i| {
        if i%r == r-1 {
            i + 1 -r
        } else {
            i + 1
        }
    };
    let up = |i| {
        if i/r == 0 {
            n - r + i
        } else {
            i - r
        }
    };
    let down = |i| {
        if i/r == c-1 {
            i%r
        } else {
            i + r
        }
    };
    let mut physical = Graph::new();
    for i in 0usize..n {
        physical.add_node(i);
    }

    for i in 0usize..n {
        physical.add_edge(NodeIndex::new(i), NodeIndex::new(left(i)), 0.0);
        physical.add_edge(NodeIndex::new(i), NodeIndex::new(right(i)), 0.0);
        physical.add_edge(NodeIndex::new(i), NodeIndex::new(up(i)), 0.0);
        physical.add_edge(NodeIndex::new(i), NodeIndex::new(down(i)), 0.0);
    }

    physical
}

fn output_result(g: &Graph<usize, f64>, p: PathBuf) {
    let dot = Dot::new(g);

    let mut type_arg = OsString::from("-T");
    if let Some(ext) = p.extension(){
        type_arg.push(ext);
    } else {
        type_arg.push("dot");
    }
    let mut graphviz = Command::new("dot")
        .arg(type_arg)
        .arg("-o")
        .arg(p)
        .stdin(Stdio::piped())
        .spawn().expect("Failed to run graphviz' dot command");
    let mut stdin = graphviz.stdin.as_mut().expect("Failed to open graphviz' dot stdin");
    write!(&mut stdin, "{:?}", dot).unwrap();
}
