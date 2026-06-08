# RUN: jit
# EXPECT: 42

def f() -> int:
    return (10 * 2 + 1) * 2
