# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: sorted()/list.sort() must dispatch __lt__ on user classes
# when no key= is given. Previously the comparator fell back to numeric
# extraction and returned Ordering::Equal for every Instance pair, so the
# list came back unchanged.

class Box:
    def __init__(self, v):
        self.v = v
    def __lt__(self, other):
        return self.v < other.v
    def __repr__(self):
        return "B(" + str(self.v) + ")"

# sorted()
print(sorted([Box(3), Box(1), Box(2)]))
print(sorted([Box(3), Box(1), Box(2)], reverse=True))

# list.sort() in place
boxes = [Box(5), Box(2), Box(4), Box(1), Box(3)]
boxes.sort()
print(boxes)

boxes.sort(reverse=True)
print(boxes)

# Mixed instance + non-instance not supported here; focus on homogeneous user-class.

# sort with key= still takes priority
print(sorted([Box(3), Box(1)], key=lambda b: -b.v))