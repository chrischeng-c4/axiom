# bool / truthiness broad

# truthy/falsy literals
print(bool(True))
print(bool(False))
print(bool(None))
print(bool(0))
print(bool(0.0))
print(bool(""))
print(bool([]))
print(bool({}))
print(bool(()))
print(bool(1))
print(bool(-1))
print(bool(1.5))
print(bool("x"))
print(bool([0]))
print(bool({"a": 1}))
print(bool((0,)))
print(bool({0}))

# short-circuit semantics
print(False or "value")
print(0 or "x")
print(None or 5)
print("" or "default")
print([] or [1])

print(True and "value")
print(1 and "x")
print(None and 5)
print("" and "default")
print([] and [1])

# chained or
print(None or None or "third")
print(None or "second" or "third")

# chained and
print("a" and "b" and "c")
print(0 and "b" and "c")
print("a" and 0 and "c")

# comparisons return bool
print(type(1 < 2).__name__)
print(type("" == "").__name__)

# ternary bool
def abs_value(n):
    return n if n >= 0 else -n

print(abs_value(5))
print(abs_value(-5))
print(abs_value(0))

# bool in set / dict
s = {True, False, 0, 1}
print(len(s))

# not on various
print(not True)
print(not False)
print(not None)
print(not 0)
print(not 1)
print(not [])
print(not [0])
