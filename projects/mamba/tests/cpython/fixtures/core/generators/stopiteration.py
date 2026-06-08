# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# StopIteration.value for return from generator
def gen_with_return():
    yield 1
    yield 2
    return "finished"

g = gen_with_return()
print(next(g))
print(next(g))
try:
    next(g)
except StopIteration as e:
    print("return value:", e.value)

# No explicit return -> StopIteration.value is None
def gen_no_return():
    yield 1

g2 = gen_no_return()
next(g2)
try:
    next(g2)
except StopIteration as e:
    print("value is None:", e.value is None)

# Return without value
def gen_bare_return():
    yield 1
    return

g3 = gen_bare_return()
next(g3)
try:
    next(g3)
except StopIteration as e:
    print("bare return value:", e.value)
