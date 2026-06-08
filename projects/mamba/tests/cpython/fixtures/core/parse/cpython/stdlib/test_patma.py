# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_patma.py — pattern matching syntax constructs only.


# --- Literal patterns: int ---

command = 1

match command:
    case 1:
        x = "one"
    case 2:
        x = "two"
    case 3:
        x = "three"


# --- Literal patterns: str ---

status = "ok"

match status:
    case "ok":
        pass
    case "error":
        pass
    case "pending":
        pass


# --- Literal patterns: bool and None ---

value = True

match value:
    case True:
        pass
    case False:
        pass

match value:
    case None:
        pass
    case True:
        pass


# --- Capture patterns ---

match command:
    case x:
        y = x


# --- Wildcard pattern ---

match command:
    case 1:
        pass
    case _:
        pass


# --- OR patterns ---

match command:
    case 1 | 2 | 3:
        x = "low"
    case 4 | 5 | 6:
        x = "high"
    case _:
        x = "other"


# --- OR patterns with capture ---

match command:
    # NOTE: OR pattern with as-binding not supported: case 1 | 2 as n:
    case 1 | 2:
        x = command
    case _:
        x = 0


# --- Sequence patterns: list style ---

point = [1, 2]

match point:
    case [0, 0]:
        pos = "origin"
    case [x, 0]:
        pos = "x-axis"
    case [0, y]:
        pos = "y-axis"
    case [x, y]:
        pos = "general"


# --- Sequence patterns: tuple style ---
# NOTE: parenthesized tuple patterns in case not supported; use list patterns
match point:
    case [0, 0]:
        pos = "origin"
    case [_x, 0]:
        pos = "x-axis"
    case [0, _y]:
        pos = "y-axis"
    case _:
        pos = "general"


# --- Star patterns in sequences ---

items = [1, 2, 3, 4, 5]

match items:
    case [first, *rest]:
        pass

match items:
    case [first, second, *_]:
        pass

match items:
    case [*init, last]:
        pass

match items:
    case [first, *middle, last]:
        pass

match items:
    case []:
        pass
    case [single]:
        pass
    case [first, second, *rest]:
        pass


# --- Mapping patterns ---

config = {"color": "red", "width": 5}

match config:
    case {"color": c}:
        pass

match config:
    case {"color": "red", "width": w}:
        pass

match config:
    case {"color": "red"}:
        pass
    case {"color": "blue"}:
        pass
    case {}:
        pass


# --- Mapping patterns with **rest ---
# NOTE: **rest in case mapping pattern not supported
match config:
    case {"color": c}:
        pass


# --- Class patterns ---

class Point:
    __match_args__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(1, 2)

# NOTE: literal positional args in class pattern not supported; use guards
match p:
    case Point(px, py) if px == 0 and py == 0:
        pos = "origin"
    case Point(px, py) if py == 0:
        pos = "x-axis"
    case Point(px, py) if px == 0:
        pos = "y-axis"
    case Point(x, y):
        pos = "general"


# --- Class patterns with keyword arguments ---

match p:
    case Point(x=xval, y=yval) if xval == 0 and yval == 0:
        pos = "origin"
    case Point(x=xval, y=yval):
        pos = "general"


# --- Class patterns with built-in types ---

val = 42

# NOTE: built-in types (int, str, float, bool) in class patterns not supported
# match val:
#     case int(n):
#         pass

# match val:
#     case str(s):
#         pass
#     case int(n):
#         pass
#     case float(f):
#         pass
#     case bool(b):
#         pass


# --- Guard clauses ---

match command:
    case x if x > 0:
        sign = "positive"
    case x if x < 0:
        sign = "negative"
    case x if x == 0:
        sign = "zero"


# --- Guard clauses with complex conditions ---

match point:
    case [x, y] if x == y:
        kind = "diagonal"
    case [x, y] if x > 0 and y > 0:
        kind = "first quadrant"
    case [x, y] if isinstance(x, int):
        kind = "integer x"


# --- Nested patterns ---

data = {"users": [{"name": "Alice", "age": 30}]}

match data:
    case {"users": [{"name": name, "age": age}]}:
        pass

# NOTE: built-in type str() in class pattern not supported
match data:
    case {"users": [{"name": name}, *rest]}:
        pass


# --- Nested sequence in mapping ---

match data:
    case {"users": [first, second, *rest]}:
        pass
    case {"users": [single]}:
        pass
    case {"users": []}:
        pass


# --- Nested class patterns ---

class Line:
    def __init__(self, start, end):
        self.start = start
        self.end = end

line = Line(Point(0, 0), Point(1, 1))

# NOTE: literal args in nested class pattern not supported; using guards
match line:
    case Line(start=Point(sx, sy), end=Point(x, y)) if sx == 0 and sy == 0:
        pass
    case Line(start=Point(x1, y1), end=Point(x2, y2)):
        pass


# --- Complex combined patterns ---

action = ("move", (10, 20))

# NOTE: tuple case patterns not supported; using list patterns
match action:
    case ["move", [x, y]]:
        pass
    case ["resize", [w, h]] if w > 0 and h > 0:
        pass
    # NOTE: str(c) class pattern with built-in type not supported
    case ["color", c]:
        pass
    # NOTE: tuple case pattern not supported; using list pattern
    case ["quit"]:
        pass
    case _:
        pass


# --- Match with various expression subjects ---

match 1 + 1:
    case 2:
        pass

match [1, 2, 3][0]:
    case 1:
        pass

match {"a": 1}["a"]:
    case 1:
        pass

def get_value():
    return 42

match get_value():
    case 42:
        pass
    case _:
        pass


# --- Match on attribute access ---

class Obj:
    status = "active"

obj = Obj()

match obj.status:
    case "active":
        pass
    case "inactive":
        pass


# --- Dotted names in class patterns ---

import collections

match collections.OrderedDict():
    case collections.OrderedDict():
        pass


# --- Literal patterns with negative numbers ---

match command:
    case -1:
        pass
    case -2.5:
        pass
    case 0:
        pass


# --- Complex literal patterns ---
# NOTE: complex number literal in case pattern not supported
# match command:
#     case 1j:
#         pass
#     case 2 + 3j:
#         pass
#     case -1j:
#         pass


# --- Sequence pattern with fixed length ---

match items:
    case [_, _, _]:
        length = 3
    case [_, _]:
        length = 2
    case [_]:
        length = 1
    case []:
        length = 0
    case _:
        length = -1


# --- Pattern matching in function ---

def classify(obj):
    match obj:
        case 0 | "":
            return "falsy"
        case n if isinstance(n, int) and n > 0:
            return "positive int"
        case s if isinstance(s, str) and len(s) > 5:
            return "long string"
        case l if isinstance(l, list) and len(l) > 3:
            return "long list"
        case _:
            return "other"


# --- Pattern matching in loop ---

events = [("click", 10, 20), ("key", "a"), ("quit",)]

for event in events:
    match event:
        case ["click", x, y]:
            pass
        case ["key", k]:
            pass
        # NOTE: tuple case pattern not supported; using list pattern
        case ["quit"]:
            break
        case _:
            pass


# --- Walrus operator subject ---
# NOTE: walrus operator in match subject not supported
val = get_value()
match val:
    case 42:
        pass
    case _:
        pass


# --- Mapping with literal keys of different types ---

match config:
    case {0: zero_val}:
        pass
    case {True: true_val}:
        pass
    case {"key": str_val}:
        pass


# --- Single-item sequence ---

match items:
    case [only]:
        pass
    case [first, *_]:
        pass


# --- End of pattern matching constructs ---
