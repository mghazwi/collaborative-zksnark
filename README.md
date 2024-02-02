# Collaborative zkSNARKs

This is a slightly modified version of the ozdemir proof-of-concept implementation of Collaborative zkSNARKs based
on Groth16, Marlin, and Plonk.
This implementation is not secure; it exists for benchmarking reasons.

This implementation is based on the paper that introduced Collaborative zkSNARKs:
["Experimenting with Collaborative zk-SNARKs: Zero-Knowledge Proofs for
Distributed Secrets"][paper].

## Starting point
Nightly rust is required. 

A good place to start is running the simple example :

1. Enter `mpc-snarks`.
3. `./run.zsh`.

[paper]: https://www.usenix.org/conference/usenixsecurity22/presentation/ozdemir