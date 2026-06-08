# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""StopIteration.value: how it is populated from a generator's return,
how it is exposed when caught manually, and how yield-from hands the
subgenerator's return value back to the delegator (PEP 380)."""


# The value attribute defaults to None, takes the first constructor
# argument, and is independently assignable.
e = StopIteration()
assert e.value is None
e = StopIteration("spam")
assert str(e) == "spam"
assert e.value == "spam"
e.value = "eggs"
assert e.value == "eggs"
print("stopiteration value attr:", "ok")


# Returning from a generator surfaces the value on StopIteration when
# the caller drives it with next() and catches the stop.
def returns_value():
    yield 1
    return ("done", 42)


g = returns_value()
assert next(g) == 1
try:
    next(g)
    raise AssertionError("expected StopIteration")
except StopIteration as stop:
    assert stop.value == ("done", 42)
print("return surfaces value:", "ok")


# yield-from binds the subgenerator's return value as the value of the
# `yield from` expression, for plain, tuple, and StopIteration payloads.
def subgen(payload):
    yield "y"
    return payload


seen = []


def delegator():
    for payload in (None, 7, (2,), StopIteration(3)):
        result = yield from subgen(payload)
        seen.append(result)


# Drain the delegator; each pass yields the single "y" then records the
# captured return value.
assert list(delegator()) == ["y", "y", "y", "y"]
assert seen[0] is None
assert seen[1] == 7
assert seen[2] == (2,)
assert isinstance(seen[3], StopIteration) and seen[3].value == 3
print("yield-from captures return:", "ok")
