# Formal Verification for Microservice Orchestration

## Getting Started

This project is only tested on Ubuntu (X86 Platform).

### Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
MSRV: 1.74

### Code Structures
- [`src/workflow`](https://github.com/ZhenbangYou/cs257-project/tree/main/src/workflow) includes a tool to allow users to express graph and schemas.
- [`src/verifier`](https://github.com/ZhenbangYou/cs257-project/blob/main/src/verifier/mod.rs) is verifier to get information about reachability, minimum input set, and robustness.

### Unit Tests and Demo
Demos and unit tests are in [`src/test`](https://github.com/ZhenbangYou/cs257-project/blob/main/tests/workflow_graph.rs). Run the following to check the result:
```
cargo test
```

### Benchmark
```
cargo bench
```

### Remarks

The values constraints for transition are not included in this prototype, but they can be added without major challenges. To add value constraints, further development could just
- Add variables for value corresponding to each relevant keys for each node.
- Enforce equality constraints where values from previous output is the same as values of current input.
- Add value constraints to transition constraints. 
