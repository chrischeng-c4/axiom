# RUN: jit
# EXPECT: 6
# Capture binding inherits the match subject type so primitive add is used (#827).

def f() -> int:
    match 5:
        case n:
            return n + 1
    return 0
