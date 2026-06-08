# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "cached_property_computes_once_per_instance"
# subject = "functools.cached_property"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cached_property: cached_property computes once per instance, keeps independent instances separate, allows manual overwrite, and works under inheritance"""
import functools


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.compute_calls = 0

    @functools.cached_property
    def magnitude_sq(self):
        self.compute_calls += 1
        return self.x * self.x + self.y * self.y


# Computed once per instance: repeated access reuses the stored value.
p = Point(3, 4)
assert p.magnitude_sq == 25, f"p.magnitude_sq = {p.magnitude_sq!r}"
assert p.magnitude_sq == 25, "p.magnitude_sq cached"
assert p.magnitude_sq == 25, "p.magnitude_sq still cached"
assert p.compute_calls == 1, f"p computed {p.compute_calls!r} times"

# A second instance is independent and does not affect p.
q = Point(5, 12)
assert q.magnitude_sq == 169, f"q.magnitude_sq = {q.magnitude_sq!r}"
assert q.compute_calls == 1, f"q computed {q.compute_calls!r} times"
assert p.compute_calls == 1, "p untouched by q"

# Manual overwrite: the descriptor only has __get__, so an explicit set
# stores a plain instance attribute that shadows the cached value.
p.magnitude_sq = 999
assert p.magnitude_sq == 999, f"p.magnitude_sq after set = {p.magnitude_sq!r}"


# cached_property is inherited and works on the subclass instance.
class Box:
    def __init__(self, n):
        self.n = n
        self.calls = 0

    @functools.cached_property
    def volume(self):
        self.calls += 1
        return self.n ** 3


class BigBox(Box):
    pass


b = BigBox(4)
assert b.volume == 64, f"b.volume = {b.volume!r}"
assert b.volume == 64, "b.volume cached"
assert b.calls == 1, f"b computed {b.calls!r} times"

print("cached_property_computes_once_per_instance OK")
