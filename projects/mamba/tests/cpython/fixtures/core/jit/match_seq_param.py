# RUN: jit
# EXPECT: 10
# Sequence pattern inside a function that receives a list parameter (#827 R5).
# Before the fix, `xs: list[int]` was typed as `int`, so mb_seq_len on the
# "int" was illegal — causing "integer out of 48-bit range" at runtime.

def first(xs: list[int]) -> int:
    match xs:
        case [x]:
            return x
    return 0

def f() -> int:
    return first([10])
