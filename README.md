# TEST
> This library is used for research purposes and is highly experimental and unstable. For real use cases, we recommend using [syn](https://github.com/QuantumHel/syn) instead.

The Experimental System for Transpilation (TEST) is an experimental system for researching quantum transpilation.

# Documentation
Due to issues with `cargo doc` use `cargo docs` instead.

## Using nighty
Install the nightly toolchain with
```
rustup toolchain install nightly
```
and then activate it in the project folder with 
```
rustup override set nightly
```

# TODO
# Simple solver target qubit adding is likely broken
## Pauli
- Nice way of handling clifford angles
## Connectivity
- See how fast can find path
- Calculate step count for routing instruction
- Edit the synthesize to accept connectivity
## Synthesize
- Merge pauli exps with same strings (should only changes 1-qubit gate count).
- Current code is written while testing things. The code eats performance for fun.

# .exp format
Used to transfer exponentials between libraries. Every line depicts one exponential as
```
angle;PAULISTRING
```
where angle is a multiple of pi (needs to be multiplied by pi).

For example
```
0.25;IIXI
-0.25;XYIZ
1.23;IYII
```