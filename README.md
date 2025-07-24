# TEST
The Experimental System for Transpilation (TEST) is an experimental system for researching quantum transpilation.

# Documentation
Due to issues with `cargo doc` use `cargo docs` instead.

# TODO
## Pauli
- Nice way of handling clifford angles
## Synthesize
- Merge pauli exps with same strings (should only changes 1-qubit gate count).
- Current code is written while testing things. The code eats performance for fun.
## Clifford Tableau
- Add solver for last qubits

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