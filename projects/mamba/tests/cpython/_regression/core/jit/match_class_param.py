# RUN: jit
# EXPECT: 43
# Class pattern inside a function that receives a class instance as parameter (#827 R5).
# Before the fix, `p: Point` was typed as `int` in HIR, so pattern dispatch
# used mb_box_int on the class pointer — breaking class matching.

class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def classify(p: Point) -> int:
    match p:
        case Point(x=a):
            return a + 1
    return 0

def f() -> int:
    p = Point(42, 0)
    return classify(p)

f()
