# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Basic generator yield
def count_up(n):
    i = 0
    while i < n:
        yield i
        i += 1

# Iterate with for loop
for x in count_up(5):
    print(x)

# Convert to list
print(list(count_up(3)))

# Manual iteration
g = count_up(3)
print(next(g))
print(next(g))
print(next(g))

# Generator exhaustion
g2 = count_up(1)
print(next(g2))
try:
    next(g2)
except StopIteration:
    print("StopIteration raised")

# Yield without value
def yield_none():
    yield
    yield
    yield

print(list(yield_none()))
