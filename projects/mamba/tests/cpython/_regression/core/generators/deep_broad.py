# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# generator deep broad

# explicit next()
def counter():
    yield 1
    yield 2
    yield 3

g = counter()
print(next(g))
print(next(g))
print(next(g))

# next with default (exhausted)
g2 = counter()
next(g2)
next(g2)
next(g2)
print(next(g2, "END"))

# yield in loop with condition
def evens_until(n):
    i = 0
    while i < n:
        if i % 2 == 0:
            yield i
        i += 1

print(list(evens_until(10)))

# yield from sub-iterable
def chained():
    yield from [10, 20]
    yield from [30, 40, 50]
    yield from (60, 70)

print(list(chained()))

# generator exhausts after first full pass
g3 = counter()
print(list(g3))
print(list(g3))

# generator with return (PEP 479-ish — normal StopIteration)
def make_gen():
    yield 1
    yield 2
    return
    yield 999  # unreachable

print(list(make_gen()))

# accumulate running state
def running_max(seq):
    m = None
    for v in seq:
        if m is None or v > m:
            m = v
        yield m

print(list(running_max([3, 1, 4, 1, 5, 9, 2, 6, 5])))

# generator in sum
def nums():
    yield 1
    yield 2
    yield 3

print(sum(nums()))

# generator used directly in for
def say():
    yield "a"
    yield "b"
    yield "c"

for s in say():
    print(s)
