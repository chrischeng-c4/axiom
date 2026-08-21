# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# generator.send() and generator.throw()

# send(value) resumes with value
def echo():
    while True:
        received = yield
        print("got:", received)

g = echo()
next(g)  # prime the generator
g.send("hello")
g.send("world")

# send returns the yielded value
def double_echo():
    val = yield "ready"
    while True:
        val = yield val * 2

g2 = double_echo()
print(next(g2))       # "ready"
print(g2.send(5))     # 10
print(g2.send(100))   # 200

# throw() injects exception
def catch_throw():
    try:
        yield 1
        yield 2
    except ValueError as e:
        print("caught:", e)
        yield 99

g3 = catch_throw()
print(next(g3))                          # 1
print(g3.throw(ValueError, "injected"))  # "caught: injected", then 99

# close() sends GeneratorExit
def closeable():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        print("generator closed")

g4 = closeable()
next(g4)
g4.close()
