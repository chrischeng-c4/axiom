# nonlocal / closure with varied types — regression coverage for the
# "box cell-variable assignments to match param-store convention" fix
# (commit a914e87e). Covers int nested chain, tuple multi-var return,
# and string accumulator. Float nonlocal is known-broken (tracked
# separately) and deliberately omitted here.

# Nested nonlocal chain: outer -> middle -> inner mutating shared int.
def nested_chain():
    x = 10
    def middle():
        nonlocal x
        def inner():
            nonlocal x
            x = x + 1
            return x
        return inner
    return middle()

f = nested_chain()
print(f())
print(f())
print(f())

# Multiple nonlocals written together, returned as a tuple.
def stats():
    count = 0
    total = 0
    def record(n):
        nonlocal count, total
        count = count + 1
        total = total + n
        return (count, total)
    return record

r = stats()
print(r(10))
print(r(20))
print(r(30))

# String nonlocal accumulator.
def joiner():
    buf = ""
    def add(s):
        nonlocal buf
        buf = buf + s
        return buf
    return add

j = joiner()
print(j("a"))
print(j("b"))
print(j("c"))
