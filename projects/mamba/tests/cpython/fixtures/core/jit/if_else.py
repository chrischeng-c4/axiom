# RUN: jit
# EXPECT: 42

def f() -> int:
    return 42 if True else 0

f()
