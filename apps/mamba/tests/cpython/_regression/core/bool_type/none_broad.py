# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# bool/None broad

# bool arithmetic
print(True + True)
print(True + False)
print(False + False)
print(True * 5)
print(False * 5)
print(True - True)
print(True - False)

# bool in comparison with int
print(True == 1)
print(False == 0)
print(True != 0)
print(False != 1)
print(True < 2)
print(True > 0)

# bool in int context
print(int(True))
print(int(False))

# int in bool context
print(bool(1))
print(bool(0))
print(bool(-1))
print(bool(42))

# string in bool context
print(bool(""))
print(bool("x"))
print(bool("False"))  # nonempty — True

# list in bool context
print(bool([]))
print(bool([0]))
print(bool([None]))

# dict in bool context
print(bool({}))
print(bool({"x": 0}))

# None
x = None
print(x)
print(x is None)
print(x is not None)
print(x == None)

# None in comparison
def maybe_get(i):
    d = {0: "a", 1: "b"}
    if i in d:
        return d[i]
    return None

print(maybe_get(0))
print(maybe_get(5))

# None as default arg
def f(x, y=None):
    if y is None:
        return x
    return x + y

print(f(1))
print(f(1, 2))

# None vs False
print(None == False)
print(None == 0)

# or / and with None
print(None or "fallback")
print(None or 0 or "x")
print(None and "never")
