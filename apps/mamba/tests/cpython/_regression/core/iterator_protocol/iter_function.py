# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/iterator_protocol: iter() idempotency, independence, and consumers."""

from operator import countOf

# iter() on an existing iterator returns that SAME iterator object
# (an iterator is its own iterator).
seq = list(range(10))
it = iter(seq)
assert iter(it) is it
print("[iter-fn] idempotent:", iter(it) is it)


# Independent iter() calls over the same container walk independently;
# nested loops produce the full Cartesian product.
EXPECTED = [(i, j) for i in range(3) for j in range(3)]
pairs = []
for i in iter(range(3)):
    for j in iter(range(3)):
        pairs.append((i, j))
assert pairs == EXPECTED
print("[iter-fn] nested-independence:", len(pairs), "pairs")


# Comprehensions drive the iterator protocol; the explicit-iter() form
# matches the implicit form.
comp_plain = [(i, j) for i in range(3) for j in range(3)]
comp_iter = [(i, j) for i in iter(range(3)) for j in iter(range(3))]
assert comp_plain == EXPECTED
assert comp_iter == EXPECTED
print("[iter-fn] comprehension-match:", comp_plain == comp_iter)


# operator.countOf consumes any iterable, counting equal elements.
assert countOf([1, 2, 2, 3, 2, 5], 2) == 3
assert countOf((1, 2, 2, 3, 2, 5), 2) == 3
assert countOf("122325", "2") == 3
assert countOf("122325", "6") == 0
d = {"a": 3, "b": 3, "c": 1}
assert countOf(d.values(), 3) == 2
print("[iter-fn] countOf:", "ok")


# A custom iterator can substitute its own values mid-stream; str.join
# consumes whatever __next__ yields.
class Substituting:
    def __init__(self, seq):
        self.inner = iter(seq)
        self.step = 0

    def __iter__(self):
        return self

    def __next__(self):
        i = self.step
        self.step = i + 1
        if i == 2:
            return "X"
        return next(self.inner)


assert " - ".join(Substituting(["a", "b", "c"])) == "a - b - X - c"
print("[iter-fn] mid-stream-substitution:", "ok")


# Container vs iterator: a container's __iter__ returns a FRESH
# iterator, so it can be looped repeatedly.
class Counter:
    def __init__(self, start, stop):
        self.i = start
        self.stop = stop

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= self.stop:
            raise StopIteration
        v = self.i
        self.i += 1
        return v


class Container:
    def __init__(self, start, stop):
        self.start = start
        self.stop = stop

    def __iter__(self):
        return Counter(self.start, self.stop)


box = Container(6, 10)
assert list(box) == [6, 7, 8, 9]
assert list(box) == [6, 7, 8, 9]
print("[iter-fn] container-repeatable:", "ok")

print("iter_function OK")
