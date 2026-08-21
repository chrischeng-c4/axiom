# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator return value is carried in StopIteration.value; yield-from receives it

def gen_returns():
    yield 1
    yield 2
    return "done"

# Manual iteration and catching StopIteration
g = gen_returns()
print(next(g))
print(next(g))
try:
    next(g)
except StopIteration as e:
    print("stopped:", e.value)

# yield-from captures the return value via assignment
def delegating():
    result = yield from gen_returns()
    yield ("result", result)

out = list(delegating())
print(out)

# Bare return (no expression) — value is None
def gen_bare():
    yield "a"
    return

g2 = gen_bare()
print(next(g2))
try:
    next(g2)
except StopIteration as e:
    print("bare:", e.value)

# Return value 0 vs None
def gen_zero():
    yield 10
    return 0

g3 = gen_zero()
print(next(g3))
try:
    next(g3)
except StopIteration as e:
    print("zero:", e.value)
