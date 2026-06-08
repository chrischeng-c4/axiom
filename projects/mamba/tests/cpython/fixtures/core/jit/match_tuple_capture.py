# RUN: jit
# EXPECT: 2
# Tuple sequence capture: case (n, _) should bind n as int so n + 1 = 2 (#827).
# Per-slot tuple typing: position 0 is int, not Union(int, int).

def f() -> int:
    match (1, 2):
        case (n, _):
            return n + 1
    return 0
