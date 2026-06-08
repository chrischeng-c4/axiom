# Regression: chained assignment `a = b = c = val` must bind every target.
# Previously the parser kept only the first target and discarded the rest,
# so later references to `b` / `c` raised "undefined name" at type-check time.

a = b = c = 10
print(a, b, c)

# Chained assign with different RHS types
x = y = z = "hello"
print(x, y, z)

# Chained assign to a list (shared reference)
p = q = [1, 2]
p.append(3)
print(q)

# Chained assign in a function (use unique names to avoid shadowing)
def make():
    aa = bb = 7
    return aa + bb

print(make())

# Four targets
a = b = c = d = 99
print(a, b, c, d)

# Chained assign evaluates RHS once — both targets see the same list
# (CPython: `a.append(x)` is visible via the other name).
u = v = []
u.append(1)
u.append(2)
print(v)
