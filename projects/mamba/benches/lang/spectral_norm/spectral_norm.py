"""spectral-norm (Computer Language Benchmarks Game) — idiomatic pure Python.

Same algorithm and same N as the sibling spectral_norm.go, so python3 / mamba
(both run THIS file) and go (the .go) are a like-for-like comparison. No numpy:
the point is "idiomatic pure Python -> native", not "Python calling C".
"""

import math

N = 1000


def eval_A(i, j):
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)


def eval_A_times_u(u):
    n = len(u)
    return [sum(eval_A(i, j) * u[j] for j in range(n)) for i in range(n)]


def eval_At_times_u(u):
    n = len(u)
    return [sum(eval_A(j, i) * u[j] for j in range(n)) for i in range(n)]


def eval_AtA_times_u(u):
    return eval_At_times_u(eval_A_times_u(u))


u = [1.0] * N
v = u
for _ in range(10):
    v = eval_AtA_times_u(u)
    u = eval_AtA_times_u(v)

vBv = sum(ue * ve for ue, ve in zip(u, v))
vv = sum(ve * ve for ve in v)
print("%.9f" % math.sqrt(vBv / vv))
