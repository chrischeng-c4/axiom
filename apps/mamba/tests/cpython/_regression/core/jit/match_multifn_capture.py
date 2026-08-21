# RUN: jit
# EXPECT: 2
# Nested capture in a function that is NOT the last lowered must still use unboxed
# value at runtime (#827). Previously only the last function's sym_types were preserved.

def helper() -> int:
    return 0

def f() -> int:
    match [1, 2]:
        case [n, _]:
            return n + 1
    return 0
