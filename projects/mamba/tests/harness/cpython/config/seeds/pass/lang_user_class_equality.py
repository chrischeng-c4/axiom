# Operational AssertionPass seed for user-defined-class equality and
# hashability. Surface: a class defining __eq__ + __hash__ has
# instances with equal field values compare ==, distinct instances
# with the same data are still `is`-distinct (no interning), the
# instances are dict-key and set-member interchangeable (a later
# assignment under p2 overwrites the value stored under p1 because
# they hash and compare equal), __eq__ against a foreign type
# returns False (via isinstance early-return), and {p1, p2, p3}
# dedupes to two members. Also covers companion dunders __repr__/
# __str__/__len__/__bool__/__contains__. Companion to lang_dunders
# and lang_dunder_protocols (which cover the dunder protocol surface
# itself, but not the collection-key use case).
_ledger: list[int] = []

class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Point):
            return False
        return self.x == other.x and self.y == other.y
    def __hash__(self) -> int:
        return hash((self.x, self.y))

p1 = Point(1, 2)
p2 = Point(1, 2)
p3 = Point(3, 4)

# Equality + reflexivity + distinct-but-equal identity
assert p1 == p2; _ledger.append(1)
assert p1 != p3; _ledger.append(1)
assert p1 == p1; _ledger.append(1)
assert hash(p1) == hash(p2); _ledger.append(1)
assert p1 is not p2; _ledger.append(1)

# Foreign-type __eq__ returns False through isinstance early-return
assert (p1 == "abc") == False; _ledger.append(1)
assert (p1 == 42) == False; _ledger.append(1)

# {p1, p2, p3} dedupes p1==p2; dict assignment under p2 overwrites p1
s = {p1, p2, p3}
assert len(s) == 2; _ledger.append(1)
d = {p1: "a", p2: "b", p3: "c"}
assert len(d) == 2; _ledger.append(1)
assert d[p1] == "b"; _ledger.append(1)

# __repr__ and __str__
class Pet:
    def __init__(self, name: str) -> None:
        self.name = name
    def __repr__(self) -> str:
        return "Pet(" + self.name + ")"
    def __str__(self) -> str:
        return "Pet:" + self.name

pet = Pet("Rex")
assert repr(pet) == "Pet(Rex)"; _ledger.append(1)
assert str(pet) == "Pet:Rex"; _ledger.append(1)

# __len__
class Box:
    def __init__(self, items: list[int]) -> None:
        self.items = items
    def __len__(self) -> int:
        return len(self.items)

assert len(Box([1, 2, 3])) == 3; _ledger.append(1)
assert len(Box([])) == 0; _ledger.append(1)

# __bool__
class Flag:
    def __init__(self, on: bool) -> None:
        self.on = on
    def __bool__(self) -> bool:
        return self.on

assert bool(Flag(True)) == True; _ledger.append(1)
assert bool(Flag(False)) == False; _ledger.append(1)

# __contains__ — `in` routes to __contains__
class Bag:
    def __init__(self, items: list[int]) -> None:
        self.items = items
    def __contains__(self, x: int) -> bool:
        return x in self.items

bag = Bag([1, 2, 3])
assert (2 in bag) == True; _ledger.append(1)
assert (10 in bag) == False; _ledger.append(1)

# Combined: __eq__ + __hash__ class works as a string-keyed dict
class K:
    def __init__(self, k: str) -> None:
        self.k = k
    def __eq__(self, other: object) -> bool:
        return isinstance(other, K) and self.k == other.k
    def __hash__(self) -> int:
        return hash(self.k)

mp = {K("a"): 1, K("b"): 2}
assert mp[K("a")] == 1; _ledger.append(1)
assert mp[K("b")] == 2; _ledger.append(1)

# Set membership through __eq__ — dedup + `in` recognize equal keys
items_set = {K("x"), K("y"), K("x")}
assert len(items_set) == 2; _ledger.append(1)
assert K("x") in items_set; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_user_class_equality {sum(_ledger)} asserts")
