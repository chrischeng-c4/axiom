# Operational AssertionPass seed for PEP 634 match-case patterns
# focusing on the `as` capture pattern (`case pat as name:`) and
# combined-pattern complexity not covered by `lang_match_class_guard`,
# `lang_match_literal_or`, `lang_match_sequence_mapping`,
# `lang_match_statement`, `lang_pep634_match_case`, or
# `lang_pep634_match_patterns`. This seed asserts: `case [pat] as
# name:` binds the matched subject to `name`; sequence-as-pattern
# captures the whole sequence (including the matched literal/wildcard
# elements); OR-pattern combined with `as` captures the matching
# branch; tuple-destructure with multiple guards distinguishes by
# value relationship; class pattern with __match_args__ delegates
# positional binding to __match_args__ order; mapping pattern
# `{"type": "circle", "radius": r}` binds nested values; mapping
# pattern with `**rest` collects leftover keys; nested sequence
# pattern destructures a list-of-lists; sequence pattern with
# `[_, _, *rest]` binds the tail.
_ledger: list[int] = []


# 1. Tuple destructure + guards
def classify_point(p):
    match p:
        case (0, 0):
            return "origin"
        case (0, y):
            return f"y-axis:{y}"
        case (x, 0):
            return f"x-axis:{x}"
        case (x, y) if x == y:
            return f"diag:{x}"
        case (x, y) if x > y:
            return f"below:{x},{y}"
        case (x, y):
            return f"above:{x},{y}"


assert classify_point((0, 0)) == "origin"; _ledger.append(1)
assert classify_point((0, 5)) == "y-axis:5"; _ledger.append(1)
assert classify_point((3, 0)) == "x-axis:3"; _ledger.append(1)
assert classify_point((4, 4)) == "diag:4"; _ledger.append(1)
assert classify_point((5, 3)) == "below:5,3"; _ledger.append(1)
assert classify_point((2, 7)) == "above:2,7"; _ledger.append(1)


# 2. Sequence-as-pattern capturing the whole list
def shape_match(seq):
    match seq:
        case [_, _, _] as triple:
            return f"triple:{triple}"
        case [_] as single:
            return f"single:{single}"
        case [] as empty:
            return f"empty:{empty}"
        case _ as other:
            return f"other:{other}"


assert shape_match([1, 2, 3]) == "triple:[1, 2, 3]"; _ledger.append(1)
assert shape_match([42]) == "single:[42]"; _ledger.append(1)
assert shape_match([]) == "empty:[]"; _ledger.append(1)
assert shape_match([1, 2]) == "other:[1, 2]"; _ledger.append(1)


# 3. OR-pattern with literal values
def small_or_big(n):
    match n:
        case 0:
            return "zero"
        case 1 | 2 | 3:
            return "small"
        case 10 | 20 | 30:
            return "medium"
        case _:
            return "other"


assert small_or_big(0) == "zero"; _ledger.append(1)
assert small_or_big(1) == "small"; _ledger.append(1)
assert small_or_big(2) == "small"; _ledger.append(1)
assert small_or_big(3) == "small"; _ledger.append(1)
assert small_or_big(20) == "medium"; _ledger.append(1)
assert small_or_big(99) == "other"; _ledger.append(1)


# 4. Class pattern with __match_args__
class _Point:
    __match_args__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y


def class_match(p):
    match p:
        case _Point(0, 0):
            return "p-origin"
        case _Point(x, 0):
            return f"p-on-x:{x}"
        case _Point(0, y):
            return f"p-on-y:{y}"
        case _Point(x, y):
            return f"p-point:{x},{y}"
        case _:
            return "not a point"


assert class_match(_Point(0, 0)) == "p-origin"; _ledger.append(1)
assert class_match(_Point(5, 0)) == "p-on-x:5"; _ledger.append(1)
assert class_match(_Point(0, 7)) == "p-on-y:7"; _ledger.append(1)
assert class_match(_Point(2, 3)) == "p-point:2,3"; _ledger.append(1)
assert class_match(42) == "not a point"; _ledger.append(1)
assert class_match("hello") == "not a point"; _ledger.append(1)


# 5. Mapping pattern — nested-value binding
def shape_desc(d):
    match d:
        case {"type": "circle", "radius": r}:
            return f"circle:{r}"
        case {"type": "square", "side": s}:
            return f"square:{s}"
        case {"type": "rect", "w": w, "h": h}:
            return f"rect:{w}x{h}"
        case {"type": t}:
            return f"unknown:{t}"
        case _:
            return "not-a-shape"


assert shape_desc({"type": "circle", "radius": 5}) == "circle:5"; _ledger.append(1)
assert shape_desc({"type": "square", "side": 3}) == "square:3"; _ledger.append(1)
assert shape_desc({"type": "rect", "w": 4, "h": 6}) == "rect:4x6"; _ledger.append(1)
assert shape_desc({"type": "triangle", "sides": 3}) == "unknown:triangle"; _ledger.append(1)
assert shape_desc([1, 2]) == "not-a-shape"; _ledger.append(1)


# 6. Mapping pattern with **rest collects leftover keys
def with_rest(d):
    match d:
        case {"name": n, **other}:
            return (n, sorted(other.keys()))
        case _:
            return None


assert with_rest({"name": "Alice", "age": 30, "city": "NYC"}) == ("Alice", ["age", "city"]); _ledger.append(1)
assert with_rest({"name": "Bob"}) == ("Bob", []); _ledger.append(1)
assert with_rest({"age": 25}) is None; _ledger.append(1)


# 7. Sequence pattern with *rest tail
def head_tail(seq):
    match seq:
        case [first, *rest]:
            return (first, rest)
        case []:
            return None


assert head_tail([1, 2, 3, 4]) == (1, [2, 3, 4]); _ledger.append(1)
assert head_tail([42]) == (42, []); _ledger.append(1)
assert head_tail([]) is None; _ledger.append(1)


def first_two_then_rest(seq):
    match seq:
        case [a, b, *rest]:
            return (a, b, rest)
        case _:
            return None


assert first_two_then_rest([1, 2, 3, 4, 5]) == (1, 2, [3, 4, 5]); _ledger.append(1)
assert first_two_then_rest([10, 20]) == (10, 20, []); _ledger.append(1)
assert first_two_then_rest([1]) is None; _ledger.append(1)


# 8. Star at end — `*init, last`
def last_with_init(seq):
    match seq:
        case [*init, last]:
            return (init, last)
        case []:
            return None


assert last_with_init([1, 2, 3, 4]) == ([1, 2, 3], 4); _ledger.append(1)
assert last_with_init([42]) == ([], 42); _ledger.append(1)
assert last_with_init([]) is None; _ledger.append(1)


# 9. Nested sequence pattern
def matrix_first_row(m):
    match m:
        case [[a, b, c], *_]:
            return (a, b, c)
        case _:
            return None


assert matrix_first_row([[1, 2, 3], [4, 5, 6]]) == (1, 2, 3); _ledger.append(1)
assert matrix_first_row([[7, 8, 9]]) == (7, 8, 9); _ledger.append(1)
assert matrix_first_row([]) is None; _ledger.append(1)
assert matrix_first_row([[1, 2]]) is None; _ledger.append(1)


# 10. Literal None / True / False / string match
def kind_of(v):
    match v:
        case None:
            return "none"
        case True:
            return "true"
        case False:
            return "false"
        case "":
            return "empty-str"
        case 0:
            return "zero"
        case _:
            return "other"


assert kind_of(None) == "none"; _ledger.append(1)
assert kind_of("") == "empty-str"; _ledger.append(1)
assert kind_of("hello") == "other"; _ledger.append(1)
assert kind_of([1]) == "other"; _ledger.append(1)


print(f"MAMBA_ASSERTION_PASS: test_match_as_pattern_complex_ops {sum(_ledger)} asserts")
