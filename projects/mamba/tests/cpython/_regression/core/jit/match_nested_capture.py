# RUN: jit
# EXPECT: 2
# Nested capture binding inside sequence pattern must unbox int before arithmetic (#827).
# match [1, 2]: case [n, _]: return n + 1  → should return 2 (not garbage).

def f() -> int:
    match [1, 2]:
        case [n, _]:
            return n + 1
    return 0
