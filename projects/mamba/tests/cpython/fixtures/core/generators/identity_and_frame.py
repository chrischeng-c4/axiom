# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Generator object identity guards: not copyable, not picklable, a
freshly created generator's frame has no caller (f_back is None), and a
generator cannot delegate into itself while it is already running."""

import copy
import pickle


def f():
    yield 1


# Generators cannot be shallow-copied; copy raises TypeError.
g = f()
try:
    copy.copy(g)
    raise AssertionError("expected TypeError from copy.copy")
except TypeError:
    pass
print("not copyable:", "ok")


# Generators cannot be pickled at any protocol.
g = f()
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    try:
        pickle.dumps(g, proto)
        raise AssertionError("expected pickling to fail")
    except (TypeError, pickle.PicklingError):
        pass
print("not picklable:", "ok")


# A generator that has not started running still owns a frame, and that
# frame has no caller frame (f_back is None).
gi = f()
assert gi.gi_frame is not None
assert gi.gi_frame.f_back is None
print("frame f_back none:", "ok")


# Attempting to `yield from` a generator that is already executing
# raises ValueError("generator already executing").
def g1():
    yield "y1"
    yield from g2()


def g2():
    yield "y2"
    yield from gi_loop  # delegate back into the running g1


gi_loop = g1()
try:
    list(gi_loop)
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert e.args[0] == "generator already executing"
print("self delegation guarded:", "ok")
