# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# yield from — full passthrough surface (R5): send, return-value capture,
# throw passthrough, close passthrough.

# send through yield-from to inner
def inner_send():
    val = yield 'ready'
    yield val * 10

def outer_send():
    result = yield from inner_send()

g1 = outer_send()
print(next(g1))       # 'ready'
print(g1.send(5))     # 50

# return value capture from inner via yield-from
def inner_return():
    yield 1
    return 42

def outer_return():
    result = yield from inner_return()
    print('got:', result)
    yield result

g3 = outer_return()
print(next(g3))   # 1
print(next(g3))   # prints 'got: 42', then yields 42

# throw passthrough — exception thrown at outer is delivered into inner
def inner_throw():
    try:
        yield 1
    except ValueError as e:
        yield 'caught:' + str(e)

def outer_throw():
    yield from inner_throw()

g4 = outer_throw()
print(next(g4))                     # 1
print(g4.throw(ValueError('x')))    # 'caught:x'

# close passthrough — close on outer must close inner; finalizers fire
# in inner-then-outer order via yield-from's GeneratorExit propagation.
finalized = []
def inner_close():
    try:
        yield 1
    finally:
        finalized.append('inner')

def outer_close():
    try:
        yield from inner_close()
    finally:
        finalized.append('outer')

g5 = outer_close()
print(next(g5))
g5.close()
print(finalized)
