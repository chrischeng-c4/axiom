# RUN: jit
# EXPECT: 42

def f() -> int:
    return (21 << 1) | 0
