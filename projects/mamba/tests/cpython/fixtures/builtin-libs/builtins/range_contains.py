# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# range.__contains__ — O(1) math check (matches CPython range_contains_long).
# Mamba represents `range()` results as iterator handles, so `value in r`
# must detect the handle and short-circuit through bounds + step alignment.

# Positive step
print(5 in range(0, 10))      # True
print(0 in range(0, 10))      # True (start inclusive)
print(9 in range(0, 10))      # True (stop-1 inclusive)
print(10 in range(0, 10))     # False (stop exclusive)
print(-1 in range(0, 10))     # False
print(15 in range(0, 10))     # False

# Step > 1
print(2 in range(0, 10, 2))   # True
print(3 in range(0, 10, 2))   # False (not on step)
print(0 in range(0, 10, 2))   # True
print(8 in range(0, 10, 2))   # True
print(10 in range(0, 10, 2))  # False (stop excl)

# Negative step (descending)
print(5 in range(10, 0, -1))  # True
print(10 in range(10, 0, -1)) # True (start inclusive)
print(0 in range(10, 0, -1))  # False (stop exclusive)
print(11 in range(10, 0, -1)) # False
print(8 in range(10, 0, -2))  # True
print(7 in range(10, 0, -2))  # False (not on step)

# Single-arg range
print(5 in range(5))          # False (5 == stop)
print(4 in range(5))          # True
print(0 in range(5))          # True

# Empty range
print(5 in range(0, 0))       # False
print(0 in range(0, 0))       # False

# `not in`
print(5 not in range(0, 10))  # False
print(15 not in range(0, 10)) # True
print(3 not in range(0, 10, 2))   # True
print(2 not in range(0, 10, 2))   # False
