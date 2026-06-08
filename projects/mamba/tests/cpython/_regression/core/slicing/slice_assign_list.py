# Regression: `lst[a:b] = [x, y]` was rejected by the type checker —
# the Index node reported element type (int) for the target, which
# didn't accept a list-of-int value. Slice indexing must return the
# container's own type so slice-assignment type-checks.

lst = [1, 2, 3, 4, 5]
print(lst[1:4])

# Simple slice replacement
lst[1:3] = [20, 30]
print(lst)

# Grow
lst[1:1] = [99, 98]
print(lst)

# Shrink
lst[0:4] = [7]
print(lst)

# Typed list via function return
def make():
    return [10, 20, 30]

xs = make()
xs[0:2] = [100, 200, 300]
print(xs)
