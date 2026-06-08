# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""yield-from delegation: next/send/close/throw routed into a subgen, with
deterministic enter/finish ordering (PEP 380). Complements
generators/yield_from_passthrough.py with full multi-level traces."""


def make_pair():
    trace = []

    def g2():
        try:
            trace.append("g2 start")
            x = yield "g2 a"
            trace.append("g2 got %s" % x)
            yield "g2 b"
        finally:
            trace.append("g2 finish")

    def g1():
        try:
            trace.append("g1 start")
            yield "g1 a"
            yield from g2()
            yield "g1 b"
        finally:
            trace.append("g1 finish")

    return trace, g1


# next() drives the whole chain; finally blocks fire inner-then-outer
# when the outer generator is fully drained. Plain iteration resumes the
# subgen's `x = yield` with None.
trace, g1 = make_pair()
assert list(g1()) == ["g1 a", "g2 a", "g2 b", "g1 b"]
assert trace == [
    "g1 start", "g2 start", "g2 got None", "g2 finish", "g1 finish",
], trace
print("delegation next:", "ok")


# close() on the delegator unwinds the active subgen first, then the
# delegator; both finally blocks run, inner before outer.
trace, g1 = make_pair()
g = g1()
assert next(g) == "g1 a"   # paused in g1
assert next(g) == "g2 a"   # now paused inside g2
g.close()
assert trace == ["g1 start", "g2 start", "g2 finish", "g1 finish"], trace
print("delegation close:", "ok")


# send() is forwarded to whichever generator is currently suspended.
trace, g1 = make_pair()
g = g1()
assert next(g) == "g1 a"
assert next(g) == "g2 a"     # suspended at `x = yield "g2 a"`
assert g.send("sent") == "g2 b"
assert "g2 got sent" in trace
print("delegation send:", "ok")


# throw() into the delegator is delivered to the subgen; uncaught there
# it propagates out, running both finally blocks on the way.
trace, g1 = make_pair()
g = g1()
next(g)
next(g)
try:
    g.throw(ValueError("ejected"))
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert e.args[0] == "ejected"
assert trace == ["g1 start", "g2 start", "g2 finish", "g1 finish"], trace
print("delegation throw:", "ok")
