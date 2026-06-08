# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Iterator protocol + StopIteration — #2796.
#
# Covers Python's iterator protocol:
#
#   __iter__    called by iter(). Must return an iterator. An
#               iterator typically returns `self`.
#   __next__    called by next() and by the `for` loop on each step.
#               Raises StopIteration when exhausted; subsequent calls
#               keep raising StopIteration (iterators are one-shot).
#
# Clauses:
#   1. for-loop consumption — `for x in it` walks until StopIteration.
#   2. Manual next() — explicit step-by-step advances match.
#   3. Exhaustion — calling next() past the end raises StopIteration
#      every subsequent time (iterator does not "reset").
#   4. next(it, default) — sentinel default returned instead of
#      raising when exhausted.
#   5. Self-returning __iter__ — iter(it) returns it itself, the
#      canonical iterator contract.
#   6. Container __iter__ returns a FRESH iterator per call — two
#      iter() calls produce independent traversals.
#
# Every print line tagged `[iterator]` so failure output names
# iterator protocol semantics.


class CountUp:
    """Minimal iterator: yields 0..stop-1, then StopIteration."""

    def __init__(self, stop):
        self.stop = stop
        self.cursor = 0
        self.stop_calls = 0

    def __iter__(self):
        # An iterator returns SELF — required by the protocol.
        return self

    def __next__(self):
        if self.cursor >= self.stop:
            self.stop_calls += 1
            raise StopIteration
        value = self.cursor
        self.cursor += 1
        return value


class Range3:
    """A CONTAINER (not iterator): __iter__ returns a fresh iterator
    each call, so two `for` loops over the same object both succeed."""

    def __iter__(self):
        return CountUp(3)


# Clause 1: for-loop consumption.
collected = [v for v in CountUp(4)]
print("[iterator] clause-1 for-loop:", collected)


# Clause 2: manual next().
it = CountUp(3)
print("[iterator] clause-2 next-0:", next(it))
print("[iterator] clause-2 next-1:", next(it))
print("[iterator] clause-2 next-2:", next(it))


# Clause 3: exhaustion — StopIteration on overshoot AND repeats.
try:
    next(it)
    print("[iterator] clause-3 first-overshoot: <unexpected-no-error>")
except StopIteration:
    print("[iterator] clause-3 first-overshoot: StopIteration")

# Second overshoot also raises StopIteration — iterators do NOT reset.
try:
    next(it)
    print("[iterator] clause-3 second-overshoot: <unexpected-no-error>")
except StopIteration:
    print("[iterator] clause-3 second-overshoot: StopIteration")

# Counter proves __next__ ran (and raised) twice past exhaustion.
print("[iterator] clause-3 stop-calls:", it.stop_calls)


# Clause 4: next(it, default) — sentinel return instead of raise.
it2 = CountUp(1)
print("[iterator] clause-4 first:", next(it2, "sentinel"))
print("[iterator] clause-4 after-end:", next(it2, "sentinel"))
print("[iterator] clause-4 still-end:", next(it2, "sentinel"))


# Clause 5: iter(iterator) returns the SAME object (self-returning).
it3 = CountUp(2)
print("[iterator] clause-5 self-iter:", iter(it3) is it3)


# Clause 6: container __iter__ returns a fresh iterator each call.
container = Range3()
pass1 = list(container)
pass2 = list(container)
print("[iterator] clause-6 pass1:", pass1)
print("[iterator] clause-6 pass2:", pass2)
# The two iterators are different objects.
i1 = iter(container)
i2 = iter(container)
print("[iterator] clause-6 distinct-iterators:", i1 is not i2)
