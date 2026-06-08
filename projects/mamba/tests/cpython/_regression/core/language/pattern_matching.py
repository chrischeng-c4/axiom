# Language conformance: pattern matching PEP 634 (R4.4).
# All 8 pattern types: literal, capture, sequence, mapping, class, OR, AS, wildcard

# --- Literal patterns ---
def classify_int(n: int) -> str:
    match n:
        case 0:
            return "zero"
        case 1:
            return "one"
        case -1:
            return "minus one"
        case _:
            return "other"

print(classify_int(0))
print(classify_int(1))
print(classify_int(-1))
print(classify_int(42))

# --- Capture patterns ---
def capture_test(val: object) -> None:
    match val:
        case x if isinstance(x, int):
            print(f"int: {x}")
        case x:
            print(f"other: {x}")

capture_test(42)
capture_test("hello")

# --- Wildcard pattern ---
def wildcard_test(val: object) -> None:
    match val:
        case 1:
            print("one")
        case _:
            print("something else")

wildcard_test(1)
wildcard_test(99)

# --- Sequence patterns ---
def seq_test(seq: object) -> None:
    match seq:
        case []:
            print("empty list")
        case [x]:
            print(f"single: {x}")
        case [x, y]:
            print(f"pair: {x}, {y}")
        case [first, *rest]:
            print(f"first={first}, rest={rest}")

seq_test([])
seq_test([1])
seq_test([1, 2])
seq_test([1, 2, 3, 4])

# --- Mapping patterns ---
def map_test(d: object) -> None:
    match d:
        case {"action": "quit"}:
            print("quit")
        case {"action": "move", "direction": direction}:
            print(f"move {direction}")
        case {"action": action}:
            print(f"action: {action}")
        case _:
            print("unknown")

map_test({"action": "quit"})
map_test({"action": "move", "direction": "north"})
map_test({"action": "fire"})
map_test({})

# --- OR patterns ---
def or_test(val: object) -> None:
    match val:
        case 1 | 2 | 3:
            print("small")
        case 10 | 20 | 30:
            print("medium")
        case _:
            print("other")

or_test(2)
or_test(20)
or_test(99)

# --- AS pattern ---
def as_test(val: object) -> None:
    match val:
        case [1, 2] as pair:
            print(f"got pair: {pair}")
        case x as captured:
            print(f"captured: {captured}")

as_test([1, 2])
as_test("hello")

# --- Class patterns ---
class Point:
    def __init__(self, x: float, y: float) -> None:
        self.x = x
        self.y = y

def classify_point(p: object) -> None:
    match p:
        case Point(x=0, y=0):
            print("origin")
        case Point(x=0, y=y):
            print(f"y-axis at {y}")
        case Point(x=x, y=0):
            print(f"x-axis at {x}")
        case Point(x=x, y=y):
            print(f"point ({x}, {y})")
        case _:
            print("not a point")

classify_point(Point(0, 0))
classify_point(Point(0, 5))
classify_point(Point(3, 0))
classify_point(Point(3, 4))

# --- Guard conditions ---
def guarded(n: int) -> None:
    match n:
        case x if x < 0:
            print(f"negative: {x}")
        case x if x == 0:
            print("zero")
        case x if x > 0:
            print(f"positive: {x}")

guarded(-5)
guarded(0)
guarded(7)
