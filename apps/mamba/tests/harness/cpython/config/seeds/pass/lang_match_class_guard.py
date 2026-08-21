# lang_match_class_guard.py - axis-1 PEP 634 match: class pattern + guard seed (#3344).
#
# Surface (from #3344):
#   1. Class pattern matches isinstance + binds fields
#   2. `case Point(x=0, y=0):` keyword field binding
#   3. `case Point(x, y) if x > y:` guard expression
#   4. `__match_args__` controls positional binding

_ledger: list[int] = []


class Point:
    __match_args__ = ("x", "y")

    def __init__(self, x, y):
        self.x = x
        self.y = y


class Origin3D:
    __match_args__ = ("x", "y", "z")

    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z


def describe(p):
    match p:
        case Point(x=0, y=0):
            return "origin"
        case Point(x, y) if x > y:
            return ("xgty", x, y)
        case Point(x, y) if x == y:
            return ("diag", x)
        case Point(x, y):
            return ("pt", x, y)
        case _:
            return "other"


def describe3d(p):
    match p:
        case Origin3D(x, y, z) if x == 0 and y == 0 and z == 0:
            return "origin3d"
        case Origin3D(x, y, z):
            return ("3d", x, y, z)


# 1. Class pattern matches isinstance + binds positional fields via __match_args__.
res_pos = describe(Point(2, 5))
assert res_pos == ("pt", 2, 5), "class pattern Point(x, y) binds via __match_args__"
_ledger.append(1)

# 1. Class pattern isinstance-gated: non-Point falls through to wildcard.
assert describe("not-a-shape") == "other", "non-matching class falls through to wildcard"
_ledger.append(1)

# 2. Keyword field binding: `Point(x=0, y=0)` matches by attribute name + literal.
assert describe(Point(0, 0)) == "origin", "keyword-field class pattern matches when fields equal literals"
_ledger.append(1)

# 3. Guard expression: `if x > y` discriminates between matching arms.
assert describe(Point(5, 3)) == ("xgty", 5, 3), "guard `if x > y` selects xgty arm"
_ledger.append(1)

# 3. Guard with equality: `if x == y` picks diagonal arm.
assert describe(Point(4, 4)) == ("diag", 4), "guard `if x == y` selects diagonal arm"
_ledger.append(1)

# 3. Guard failure falls through to next matching arm.
assert describe(Point(1, 4)) == ("pt", 1, 4), "guard failure falls through to unconditional Point arm"
_ledger.append(1)

# 4. __match_args__ ordering controls positional binding (3-arg variant).
assert describe3d(Origin3D(0, 0, 0)) == "origin3d", "3-arg __match_args__ binds all positions for guard"
_ledger.append(1)

assert describe3d(Origin3D(7, 8, 9)) == ("3d", 7, 8, 9), "3-arg __match_args__ binds in declared order"
_ledger.append(1)

# 4. Two classes with different __match_args__ can be distinguished in same statement.
def kind(p):
    match p:
        case Point(_, _):
            return "Point"
        case Origin3D(_, _, _):
            return "Origin3D"
        case _:
            return "?"

assert kind(Point(0, 0)) == "Point", "class pattern distinguishes Point from Origin3D"
_ledger.append(1)

assert kind(Origin3D(0, 0, 0)) == "Origin3D", "class pattern distinguishes Origin3D from Point"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_match_class_guard {sum(_ledger)} asserts")
