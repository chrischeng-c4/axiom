# Multi-value return and tuple unpacking

def swap(a, b):
    return b, a

x, y = swap(1, 2)
print(x, y)

# Return tuple
def minmax(xs):
    return min(xs), max(xs)

lo, hi = minmax([3, 1, 4, 1, 5, 9, 2, 6])
print(lo, hi)

# Three values
def stats(xs):
    return len(xs), sum(xs), sum(xs) / len(xs)

n, s, avg = stats([1, 2, 3, 4, 5])
print(n, s, avg)

# Starred unpacking
def multi():
    return 1, 2, 3, 4, 5

first, *rest = multi()
print(first)
print(rest)

*init, last = multi()
print(init)
print(last)

head, *mid, tail = multi()
print(head)
print(mid)
print(tail)

# Unpack in for-loop
pairs = [(1, "a"), (2, "b"), (3, "c")]
for n, s in pairs:
    print(n, s)

# Swap via tuple
a = 10
b = 20
a, b = b, a
print(a, b)

