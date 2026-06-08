# match / case pattern semantics — #2798.
#
# Covers Python 3.10+ structural pattern matching (PEP 634) basics.
# Not an exhaustive PEP 634 conformance run — only the acceptance
# points pinned by #2798.
#
# Patterns covered:
#   - literal pattern        case 0: / case "a":
#   - capture pattern        case x:
#   - wildcard               case _:
#   - sequence pattern       case [a, b, *rest]:
#   - mapping pattern        case {"k": v}:
#   - class pattern          case Point(x=0, y=y):
#   - or-pattern             case 1 | 2 | 3:
#   - guard                  case x if x > 0:
#
# Clauses:
#   1. Literal + capture + wildcard pick the correct case and bind
#      the expected variable.
#   2. Sequence pattern destructures [a, b, *rest].
#   3. Mapping pattern matches dict subset and binds keyed value.
#   4. Class pattern with positional + keyword sub-patterns binds
#      attributes.
#   5. Or-pattern matches any alternative.
#   6. Guard rejects an otherwise-matching case; falls through to
#      next.
#
# Every print line tagged `[match]` so failure output names pattern
# matching semantics. Fixture is NOT pre-marked xfail; if the
# runtime does not support `match`, parsing fails loudly per the
# acceptance text.


from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int


def classify_literal(v):
    match v:
        case 0:
            return ("zero",)
        case "a":
            return ("string-a",)
        case [1, 2]:
            return ("seq-1-2",)
        case x if isinstance(x, int) and x > 0:
            return ("positive", x)
        case _:
            return ("wildcard",)


def destructure_seq(seq):
    match seq:
        case []:
            return ("empty",)
        case [a]:
            return ("one", a)
        case [a, b]:
            return ("two", a, b)
        case [a, b, *rest]:
            return ("rest", a, b, rest)
        case _:
            return ("wildcard",)


def lookup_mapping(m):
    match m:
        case {"name": name, "age": age}:
            return ("name-age", name, age)
        case {"name": name}:
            return ("name-only", name)
        case {}:
            return ("empty-map",)
        case _:
            return ("wildcard",)


def describe_point(p):
    match p:
        case Point(x=0, y=0):
            return ("origin",)
        case Point(x=0, y=y):
            return ("y-axis", y)
        case Point(x=x, y=0):
            return ("x-axis", x)
        case Point(x=x, y=y):
            return ("general", x, y)
        case _:
            return ("not-point",)


def or_pattern(v):
    match v:
        case 1 | 2 | 3:
            return ("low",)
        case "yes" | "no":
            return ("boolean-word",)
        case _:
            return ("other",)


def guard_demo(v):
    match v:
        case x if x < 0:
            return ("negative", x)
        case 0:
            return ("zero",)
        case x if x > 100:
            return ("huge", x)
        case x:
            return ("small-positive", x)


# Clause 1: literal + capture + wildcard.
print("[match] clause-1 zero:", classify_literal(0))
print("[match] clause-1 string:", classify_literal("a"))
print("[match] clause-1 seq:", classify_literal([1, 2]))
print("[match] clause-1 positive:", classify_literal(7))
print("[match] clause-1 wild:", classify_literal(3.14))


# Clause 2: sequence destructuring.
print("[match] clause-2 empty:", destructure_seq([]))
print("[match] clause-2 one:", destructure_seq([10]))
print("[match] clause-2 two:", destructure_seq([10, 20]))
print("[match] clause-2 rest:", destructure_seq([10, 20, 30, 40]))


# Clause 3: mapping pattern.
print("[match] clause-3 name-age:", lookup_mapping({"name": "ada", "age": 36}))
print("[match] clause-3 name-only:", lookup_mapping({"name": "tim"}))
# A non-empty map without "name" still matches the {} case-pattern
# because {} matches ANY mapping (the empty subset is always a
# subset). This is PEP 634 behavior.
print("[match] clause-3 unrelated-key:", lookup_mapping({"other": 1}))


# Clause 4: class pattern.
print("[match] clause-4 origin:", describe_point(Point(0, 0)))
print("[match] clause-4 y-axis:", describe_point(Point(0, 7)))
print("[match] clause-4 x-axis:", describe_point(Point(7, 0)))
print("[match] clause-4 general:", describe_point(Point(3, 4)))


# Clause 5: or-pattern.
print("[match] clause-5 low-1:", or_pattern(1))
print("[match] clause-5 low-3:", or_pattern(3))
print("[match] clause-5 yes:", or_pattern("yes"))
print("[match] clause-5 other:", or_pattern("nope"))


# Clause 6: guard rejects, falls through.
print("[match] clause-6 neg:", guard_demo(-5))
print("[match] clause-6 zero:", guard_demo(0))
print("[match] clause-6 huge:", guard_demo(500))
print("[match] clause-6 small:", guard_demo(42))
