# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator send edge cases (R2).

# send(None) primes the generator — same as next()
def gen1():
    val = yield 1
    yield val * 2

g = gen1()
print(g.send(None))  # same as next(g): yields 1

# send(value) returns next yielded value
print(g.send(5))  # yields 10

# send to exhausted generator raises StopIteration
def gen2():
    yield 1

g2 = gen2()
next(g2)
try:
    next(g2)
except StopIteration:
    print('exhausted')

try:
    g2.send(1)
except StopIteration:
    print('send to exhausted')

# send(non-None) to a just-started generator raises TypeError
def gen3():
    val = yield 1
    yield val

g3 = gen3()
try:
    g3.send(5)
except TypeError as e:
    print('TypeError:', e)
