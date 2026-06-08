# RUN: jit
# EXPECT: 42

def f() -> int:
    return (2 + 5) * 6
