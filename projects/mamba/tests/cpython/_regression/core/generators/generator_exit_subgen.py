# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""GeneratorExit thrown into a delegating generator reaches the active
subgenerator; how the subgen reacts decides the outcome (PEP 380)."""


# Subgen catches GeneratorExit and returns -> GeneratorExit re-raised
# out of the delegator (a returning subgen does not suppress it).
trace = []


def sub_returns():
    try:
        trace.append("enter")
        yield
    except GeneratorExit:
        return


def deleg_returns():
    yield from sub_returns()


g = deleg_returns()
next(g)
try:
    g.throw(GeneratorExit)
    raise AssertionError("expected GeneratorExit")
except GeneratorExit:
    pass
assert trace == ["enter"], trace
print("subgen returns:", "ok")


# Subgen catches GeneratorExit and raises a different exception ->
# that exception propagates, chained from the GeneratorExit.
trace = []


def sub_raises():
    try:
        trace.append("enter")
        yield
    except GeneratorExit:
        raise ValueError("converted")


def deleg_raises():
    yield from sub_raises()


g = deleg_raises()
next(g)
try:
    g.throw(GeneratorExit)
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert e.args[0] == "converted"
    assert isinstance(e.__context__, GeneratorExit)
assert trace == ["enter"], trace
print("subgen raises:", "ok")


# Subgen catches GeneratorExit and yields again -> illegal; the runtime
# converts it to RuntimeError("generator ignored GeneratorExit").
trace = []


def sub_yields():
    try:
        trace.append("enter")
        yield
    except GeneratorExit:
        yield


def deleg_yields():
    yield from sub_yields()


g = deleg_yields()
next(g)
try:
    g.throw(GeneratorExit)
    raise AssertionError("expected RuntimeError")
except RuntimeError as e:
    assert e.args[0] == "generator ignored GeneratorExit"
assert trace == ["enter"], trace
print("subgen yields:", "ok")
