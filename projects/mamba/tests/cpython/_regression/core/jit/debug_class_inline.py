# RUN: jit
# EXPECT: 43
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    match p:
        case Point(x=a):
            return a + 1
    return 0

f()
