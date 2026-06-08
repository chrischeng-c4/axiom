# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""throw() into a delegating generator where the subgenerator CATCHES
the exception: it may resume yielding, or return a value that yield-from
hands back to the delegator (PEP 380)."""


# Subgen catches the thrown exception and keeps yielding; control stays
# inside the subgen, then naturally returns to the delegator.
trace = []


def sub_resumes():
    try:
        yield "a"
        yield "b"  # skipped: throw lands while paused at "a"
    except ValueError:
        trace.append("caught")
        yield "recovered"


def deleg_resumes():
    yield from sub_resumes()
    yield "outer tail"


g = deleg_resumes()
assert next(g) == "a"
assert g.throw(ValueError("x")) == "recovered"
assert next(g) == "outer tail"
assert trace == ["caught"], trace
print("delegated throw resumes:", "ok")


# Subgen catches the thrown exception and RETURNS a value; yield-from
# binds that value, and it works for plain / tuple / StopIteration
# payloads alike.
def sub_returns(value):
    try:
        yield 1
    except ValueError:
        return value


def deleg_returns(value):
    captured = yield from sub_returns(value)
    yield ("captured", captured)


for payload in (2, (2,), StopIteration(2)):
    g = deleg_returns(payload)
    assert next(g) == 1
    tag, got = g.throw(ValueError)
    assert tag == "captured"
    assert got is payload
print("delegated throw returns:", "ok")
