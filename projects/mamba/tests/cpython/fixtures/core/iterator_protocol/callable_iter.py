# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/iterator_protocol: the two-argument iter(callable, sentinel) form."""

# iter(callable, sentinel) builds an iterator that calls `callable()`
# with no arguments on each step and stops when the result equals the
# sentinel (the sentinel value itself is NOT yielded).
def make_counter():
    state = {"n": 0}

    def step():
        n = state["n"]
        state["n"] = n + 1
        return n

    return step


it = iter(make_counter(), 5)
assert list(it) == [0, 1, 2, 3, 4]
print("[callable-iter] stops-before-sentinel:", "ok")

# The iterator is one-shot: once the sentinel has been seen it stays
# exhausted, even though the underlying callable could keep producing.
assert list(it) == []
print("[callable-iter] one-shot:", "ok")


# An exception raised by the callable propagates straight out of the
# loop; values produced before the failure are kept.
def make_failing():
    state = {"n": 0}

    def step():
        n = state["n"]
        state["n"] = n + 1
        if n == 3:
            raise RuntimeError("boom")
        return n

    return step


collected = []
try:
    for value in iter(make_failing(), 99):
        collected.append(value)
    raised = False
except RuntimeError:
    raised = True
assert raised
assert collected == [0, 1, 2]
print("[callable-iter] exception-propagates:", collected)


# Reentrant exhaustion is concealed: if the callable recursively drains
# its own iterator, the outer next() still terminates with
# StopIteration rather than looping forever.
HAS_MORE, NO_MORE = 1, 2


def recursive():
    if recursive.reentered:
        return NO_MORE
    recursive.reentered = True
    list(recursive.iterator)  # drain from inside the callable
    return HAS_MORE


recursive.reentered = False
recursive.iterator = iter(recursive, NO_MORE)
try:
    next(recursive.iterator)
    reentrant_stopped = False
except StopIteration:
    reentrant_stopped = True
assert reentrant_stopped
print("[callable-iter] reentrant-exhaustion:", "StopIteration")

print("callable_iter OK")
