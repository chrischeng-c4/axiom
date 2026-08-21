# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator close edge cases (R4).

# close() on unstarted generator — no-op
def gen1():
    yield 1
    yield 2

g1 = gen1()
g1.close()
print('unstarted close: ok')

# close() triggers GeneratorExit in active generator
def gen2():
    try:
        yield 1
    except GeneratorExit:
        print('GeneratorExit caught')

g3 = gen2()
next(g3)
g3.close()

# close() triggers finally block
def gen3():
    try:
        yield 1
        yield 2
    finally:
        print('finally ran')

g4 = gen3()
next(g4)
g4.close()

# close() on exhausted generator — no-op (no exception, no resume)
def gen4():
    yield 1

g5 = gen4()
list(g5)  # exhaust
g5.close()
print('close-after-exhaust: ok')

# Generator that swallows GeneratorExit and yields again must surface
# RuntimeError("generator ignored GeneratorExit") to the close() caller.
def gen5():
    try:
        yield 1
    except GeneratorExit:
        yield 2  # illegal — close must convert this to RuntimeError

g6 = gen5()
next(g6)
try:
    g6.close()
except RuntimeError as e:
    print('RuntimeError:', e)
