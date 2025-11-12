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
0.25;XIIX
a;IIXI
-a;XIII
0.36;IIIZ
-0.5;IIXI

```