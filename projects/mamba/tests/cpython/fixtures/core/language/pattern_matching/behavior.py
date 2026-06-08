"""Behavior contract for language pattern matching (match/case, PEP 634).

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: match evaluates the subject exactly once
_evals = []
def _subject(v: int) -> int:
    _evals.append(v)
    return v

match _subject(42):
    case 42:
        _hit = True
    case _:
        _hit = False
assert _hit, "literal 42 not matched"
assert _evals == [42], f"subject evaluated {len(_evals)} times"

# Rule 2: First matching case wins (no fall-through)
def _match_first(n: int) -> str:
    match n:
        case 1:
            return "first"
        case 1:  # unreachable  # noqa: F811
            return "second"
        case _:
            return "other"

assert _match_first(1) == "first", "first case should win"

# Rule 3: Wildcard _ matches anything and doesn't bind
_wild = None
match 99:
    case _:
        _wild = "matched"
assert _wild == "matched", f"wildcard = {_wild!r}"

# Rule 4: Capture pattern binds the subject
match [1, 2, 3]:
    case [a, b, c]:
        _bound = (a, b, c)
assert _bound == (1, 2, 3), f"sequence capture = {_bound!r}"

# Rule 5: Guard — only match when guard is True
def _guarded(n: int) -> str:
    match n:
        case x if x % 2 == 0:
            return "even"
        case x:
            return "odd"

assert _guarded(4) == "even", f"4 = {_guarded(4)!r}"
assert _guarded(7) == "odd", f"7 = {_guarded(7)!r}"

# Rule 6: Sequence pattern with *rest capture
match [10, 20, 30, 40]:
    case [first, *rest]:
        _seq_first = first
        _seq_rest = rest

assert _seq_first == 10, f"seq_first = {_seq_first!r}"
assert _seq_rest == [20, 30, 40], f"seq_rest = {_seq_rest!r}"

# Rule 7: Mapping pattern — only named keys need to match, extras allowed
match {"x": 1, "y": 2, "z": 3}:
    case {"x": xv, "y": yv}:
        _map_result = (xv, yv)

assert _map_result == (1, 2), f"map result = {_map_result!r}"

# Rule 8: OR pattern — any alternative matches
def _color(c: str) -> str:
    match c:
        case "red" | "crimson":
            return "red-family"
        case "blue" | "navy":
            return "blue-family"
        case _:
            return "other"

assert _color("red") == "red-family", f"red = {_color('red')!r}"
assert _color("crimson") == "red-family", f"crimson = {_color('crimson')!r}"
assert _color("navy") == "blue-family", f"navy = {_color('navy')!r}"
assert _color("green") == "other", f"green = {_color('green')!r}"

# Rule 9: Class pattern
class _Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

def _describe_point(p) -> str:
    match p:
        case _Point(x=0, y=0):
            return "origin"
        case _Point(x=0, y=_y):
            return f"y-axis:{_y}"
        case _Point(x=_x, y=0):
            return f"x-axis:{_x}"
        case _Point(x=_x, y=_y):
            return f"point:{_x},{_y}"

assert _describe_point(_Point(0, 0)) == "origin", f"origin = {_describe_point(_Point(0,0))!r}"
assert _describe_point(_Point(0, 5)) == "y-axis:5", f"y-axis = {_describe_point(_Point(0,5))!r}"
assert _describe_point(_Point(3, 4)) == "point:3,4", f"point = {_describe_point(_Point(3,4))!r}"

# Rule 10: No match → falls through (no case hits)
_result = "default"
match 999:
    case 1:
        _result = "one"
    case 2:
        _result = "two"
assert _result == "default", f"no match = {_result!r}"

print("behavior OK")
