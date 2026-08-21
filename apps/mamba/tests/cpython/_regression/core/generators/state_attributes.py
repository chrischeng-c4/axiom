# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator state — basic lifecycle

def gen():
    yield 1
    yield 2

g = gen()

# next returns first value
print(next(g))

# Exhaust the generator
print(next(g))

# StopIteration after exhaustion
try:
    next(g)
except StopIteration:
    print('exhausted')

# close on active generator
g2 = gen()
next(g2)
g2.close()
print('closed ok')
