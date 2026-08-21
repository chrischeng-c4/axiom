# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: min()/max() must dispatch __lt__ on user-class values.
# Previously compare_values was int/str/numeric-only and returned `<`
# via unwrap_or(0.0) for Instance operands, so min/max on a list of
# user-class values almost always returned the wrong element.

class Box:
    def __init__(self, v):
        self.v = v
    def __lt__(self, other):
        return self.v < other.v
    def __repr__(self):
        return "B(" + str(self.v) + ")"

boxes = [Box(3), Box(1), Box(4), Box(1), Box(5)]

# min / max over a list
print(min(boxes))
print(max(boxes))

# min / max over varargs
print(min(Box(2), Box(5), Box(1)))
print(max(Box(2), Box(5), Box(1)))

# with key=
print(min(boxes, key=lambda b: -b.v))
print(max(boxes, key=lambda b: -b.v))

# default= on empty iterable
print(min([], default=Box(99)))
print(max([], default=Box(99)))

# Preserve the existing int / str behavior (guard against regression
# in the simpler path).
print(min([3, 1, 4, 1, 5, 9, 2, 6]))
print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(min(["banana", "apple", "cherry"]))
print(max(["banana", "apple", "cherry"]))
