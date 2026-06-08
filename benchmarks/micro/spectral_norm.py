# Benchmark: Spectral norm (simplified, n=100).
# Measures: matrix-vector product, floating-point precision.

def A(i: int, j: int) -> float:
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)


def multiply_Av(n: int, v: list) -> list:
    result: list = [0.0] * n
    for i in range(n):
        s: float = 0.0
        for j in range(n):
            s += A(i, j) * v[j]
        result[i] = s
    return result


def multiply_Atv(n: int, v: list) -> list:
    result: list = [0.0] * n
    for i in range(n):
        s: float = 0.0
        for j in range(n):
            s += A(j, i) * v[j]
        result[i] = s
    return result


def multiply_AtAv(n: int, v: list) -> list:
    return multiply_Atv(n, multiply_Av(n, v))


n: int = 100
u: list = [1.0] * n
for _i in range(10):
    v = multiply_AtAv(n, u)
    u = multiply_AtAv(n, v)

vBv: float = 0.0
vv: float = 0.0
for i in range(n):
    vBv += u[i] * v[i]
    vv += v[i] * v[i]

import math
result: float = (vBv / vv) ** 0.5
print(result)
