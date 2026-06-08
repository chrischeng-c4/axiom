# Regression: `v = f()` at module scope where f returns an int, followed
# by `print(v)`. The checker recorded f's non-annotated return as Any
# while the HIR layer inferred int from the return literal; the mismatch
# left v with checker-type Any, the boxing step at print's arg skipped
# NaN-boxing, and the int leaked as raw bits.

def f():
    return 42

v = f()
print(v)

# Same with str / float / list returns (these already worked — guard)
def g():
    return "hello"

s = g()
print(s)

def h():
    return 3.14

x = h()
print(x)

def make():
    return [1, 2, 3]

xs = make()
print(xs)

# Multiple int-returning calls
def square(n):
    return n * n

a = square(5)
b = square(6)
print(a, b, a + b)
