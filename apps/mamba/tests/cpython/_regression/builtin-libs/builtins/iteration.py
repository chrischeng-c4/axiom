# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: iteration utilities (R1.7).
# iter, next, all, any — exhaustion, short-circuit, StopIteration

# iter from list
it = iter([10, 20, 30])
print(next(it))
print(next(it))
print(next(it))

# next with default (no StopIteration)
it2 = iter([1])
print(next(it2, None))
print(next(it2, None))

# StopIteration from next without default
it3 = iter([])
try:
    next(it3)
except StopIteration:
    print("StopIteration raised")

# all — short-circuits on first False
print(all([True, True, True]))
print(all([True, False, True]))
print(all([]))        # vacuously True
print(all([0, 1, 2]))

# any — short-circuits on first True
print(any([False, False, True]))
print(any([False, False, False]))
print(any([]))        # vacuously False
print(any([0, 0, 1]))

# iter over string
chars = list(iter("abc"))
print(chars)

# iter with sentinel (callable form)
vals = [3, 2, 1, 0]
idx = 0
def next_val() -> int:
    global idx
    v = vals[idx]
    idx += 1
    return v

sentinel_iter = iter(next_val, 0)
result = list(sentinel_iter)
print(result)
