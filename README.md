# An optimizer for the Logical Topology Design problem

This program implement a solver of the LTD problem in the Rust programming language.

It uses heuristic algorithms to find the best Logical topology, either on a random physical topology, on an opportunistic physical topology, or on a given regular topology (Manhattan).

It makes some use of mutlithreading, when it was not too difficult to add it.

# Complexity
The computational complexity of this program is **O(E + N log N)** in the worst case to compute the logical topology when splitting the traffic is not allowed, or else **O((N/3)*(N*D+N log N))** if it is allowed, **N** being the number of nodes in the network and **E** the number of links in the physical topology.

# Usage
To compile this package, ensure you have a recent version of the Rust compiler and Cargo, the Rust package manager, installed and run
`cargo build --release`
Then type
`./ltd --help`
in the target directory or 
`cargo run --release -- --help`
in any of the program directories to see all the possible options that it can accept.

## Credits
It has been developed together by:
* Silvia Bova
* Gianmarco Garrisi
* Nicolò Macaluso
* Ruben Monti
