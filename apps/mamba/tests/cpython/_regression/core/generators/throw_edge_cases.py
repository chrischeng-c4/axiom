# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator throw edge cases

# throw with no matching except — propagates to caller
def gen1():
    yield 1
    yield 2

g1 = gen1()
next(g1)
try:
    g1.throw(TypeError('bad'))
except TypeError as e:
    print('propagated:', e)

# throw into generator with matching except
def gen2():
    try:
        yield 1
        yield 2
    except ValueError as e:
        print('caught:', e)
        yield 99

g2 = gen2()
print(next(g2))
print(g2.throw(ValueError('injected')))

# throw into finally block
def gen3():
    try:
        yield 1
    finally:
        print('cleanup')

g3 = gen3()
next(g3)
try:
    g3.throw(ValueError('error'))
except ValueError:
    print('ValueError propagated after cleanup')

# throw on exhausted generator
def gen4():
    yield 1

g4 = gen4()
next(g4)
try:
    next(g4)
except StopIteration:
    pass
try:
    g4.throw(RuntimeError('late throw'))
except RuntimeError as e:
    print('exhausted throw:', e)
