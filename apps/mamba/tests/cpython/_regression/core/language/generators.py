# Language conformance: generator full protocol (R4.7).
# yield, yield from, send(), throw(), close(), StopIteration.value
# Async generators: marked xfail

# --- Basic yield ---
def counter(n: int) -> object:
    for i in range(n):
        yield i

print(list(counter(5)))

# --- yield from ---
def chain(*iterables: object) -> object:
    for it in iterables:
        yield from it  # type: ignore[misc]

print(list(chain([1, 2], [3, 4], [5])))

# --- send() ---
def accumulator() -> object:
    total = 0
    while True:
        value = yield total
        if value is None:
            break
        total += value  # type: ignore[operator]

gen = accumulator()
print(next(gen))      # prime: 0
print(gen.send(10))   # 10
print(gen.send(20))   # 30
print(gen.send(5))    # 35

# --- throw() ---
def safe_gen() -> object:
    try:
        while True:
            yield "running"
    except ValueError as e:
        yield f"caught: {e}"
        return

g = safe_gen()
print(next(g))
print(g.throw(ValueError, "oops"))

# --- close() ---
def infinite() -> object:
    try:
        while True:
            yield 1
    except GeneratorExit:
        print("generator closed")

inf = infinite()
next(inf)
inf.close()

# --- StopIteration.value ---
def return_value() -> object:
    yield 1
    yield 2
    return "done"

def wrapper() -> object:
    result = yield from return_value()
    print(f"return value: {result}")

print(list(wrapper()))

# --- Generator state: created, running, suspended, closed ---
def simple() -> object:
    yield 1
    yield 2

sg = simple()
print(sg.gi_frame is not None)   # suspended (not yet started, but frame exists)
next(sg)
next(sg)
try:
    next(sg)
except StopIteration:
    pass
print(sg.gi_frame is None)       # closed

# --- Async generators: xfail — not yet implemented ---
# # mamba-xfail: async generators not yet implemented (see #800)
# async def async_gen():
#     for i in range(3):
#         yield i
