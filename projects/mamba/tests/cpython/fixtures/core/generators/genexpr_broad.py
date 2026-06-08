# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator expressions — broader coverage: any/all, set/tuple
# materialization, and for-loop consumption.
# (Basic cases already in generators/genexpr.py.)

# passed to any / all
print(any(x > 3 for x in [1, 2, 3, 4]))
print(any(x > 10 for x in [1, 2, 3, 4]))
print(all(x > 0 for x in [1, 2, 3, 4]))
print(all(x > 2 for x in [1, 2, 3, 4]))

# passed to sorted
print(sorted(x * -1 for x in [3, 1, 4, 1, 5]))

# passed to tuple / list / set
print(tuple(x * 2 for x in range(3)))
print(list(x + 1 for x in range(3)))
print(sorted(set(x % 3 for x in range(10))))

# consumed by for-loop
total = 0
for v in (x * 2 for x in range(5)):
    total += v
print(total)

# genexpr over string
print(list(c.upper() for c in "abc"))
print("".join(c for c in "hello" if c != "l"))

# genexpr with conditional expression
print(list(("big" if x > 2 else "small") for x in range(5)))

# sum-like patterns
print(sum(x * x for x in range(1, 5)))
