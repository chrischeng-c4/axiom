# Regression: user classes with __hash__ + __eq__ can serve as dict keys
# and set elements. Previously DictKey::Other keyed by pointer-identity
# (stringified NaN-box bits), so equal-but-distinct Coord instances never
# collided — dict[Coord(1,2)] lookups returned None, and set/frozenset
# dedup left all duplicates intact.

class Coord:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __hash__(self):
        return hash((self.x, self.y))
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y
    def __repr__(self):
        return "C(" + str(self.x) + "," + str(self.y) + ")"

# dict lookup through equal-but-distinct keys
d = {Coord(1, 2): "a", Coord(3, 4): "b"}
print(d[Coord(1, 2)])
print(d[Coord(3, 4)])
print(Coord(1, 2) in d)
print(Coord(5, 6) in d)

# Reassignment through an equal instance collapses to one entry
d[Coord(1, 2)] = "updated"
print(len(d))
print(d[Coord(1, 2)])

# Set dedup
s = {Coord(1, 2), Coord(3, 4), Coord(1, 2)}
print(len(s))
print(Coord(3, 4) in s)
print(Coord(5, 6) in s)

# Frozenset dedup
fs = frozenset([Coord(1, 2), Coord(1, 2), Coord(3, 4)])
print(len(fs))

# Set operations on user-class elements
s1 = {Coord(1, 2), Coord(3, 4)}
s2 = {Coord(3, 4), Coord(5, 6)}
print(len(s1 & s2))
print(len(s1 | s2))
print(len(s1 - s2))
