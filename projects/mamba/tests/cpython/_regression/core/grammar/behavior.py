# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert isinstance(1, int)
assert type(()) is tuple
assert type([]) is list
assert type({}) is dict
assert len("abc") == 3
assert list(range(3)) == [0, 1, 2]
print("basics: ok")

# del with nested-target unpacking removes every named binding.
a, b, c, d, e, f, g = "abcdefg"
del a, (b, c), (d, (e, f))
_gone = [n for n in ("a", "b", "c", "d", "e", "f") if n not in globals()]
assert _gone == ["a", "b", "c", "d", "e", "f"], _gone
assert g == "g"  # untouched
# del of a list slice mutates in place.
items = list("abcd")
del items[1:3]
assert items == ["a", "d"]
print("del_stmt: ok")

# assert with a message: the message becomes args[0]; bare assert has no args.
try:
    assert 0, "boom"
    raise AssertionError("assert 0 should have failed")
except AssertionError as exc:
    assert exc.args[0] == "boom"
try:
    assert False
    raise AssertionError("assert False should have failed")
except AssertionError as exc:
    assert len(exc.args) == 0
print("assert_stmt: ok")

# Comprehension iterable is evaluated eagerly at genexpr creation;
# the inner reference to a rebindable name is captured lazily.
x = 10
gen = (i for i in range(x))
x = 5
assert len(list(gen)) == 10            # range(10) was bound at creation time
assert [v for v in range(10) if v % 2 if v % 3] == [1, 5, 7]
assert [v for (v,) in [(4,), (5,), (6,)]] == [4, 5, 6]
print("comprehension_eval: ok")

print("behavior OK")
