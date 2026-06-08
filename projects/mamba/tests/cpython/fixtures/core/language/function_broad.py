def greet(name, greeting="hello"):
    return f"{greeting}, {name}!"

print(greet("world"))
print(greet("claude", "hi"))
print(greet("py", greeting="hola"))

def func(a, b, c):
    return a * 100 + b * 10 + c

print(func(1, 2, 3))
print(func(a=1, b=2, c=3))
print(func(1, c=3, b=2))
print(func(1, 2, c=3))

def sum_all(*args):
    total = 0
    for x in args:
        total += x
    return total

print(sum_all())
print(sum_all(1, 2, 3))
print(sum_all(1, 2, 3, 4, 5, 6, 7))

def describe(**kwargs):
    parts = []
    for k in sorted(kwargs.keys()):
        parts.append(f"{k}={kwargs[k]}")
    return ", ".join(parts)

print(describe(x=1, y=2, z=3))
print(describe(name="Alice", age=30))

# recursion
def fact(n):
    if n <= 1:
        return 1
    return n * fact(n - 1)

print(fact(5))
print(fact(10))

# mutual recursion
def is_even(n):
    if n == 0:
        return True
    return is_odd(n - 1)

def is_odd(n):
    if n == 0:
        return False
    return is_even(n - 1)

print(is_even(10))
print(is_odd(7))

# closure
def make_counter():
    n = [0]
    def inc():
        n[0] += 1
        return n[0]
    return inc

c = make_counter()
print(c())
print(c())
print(c())

# lambda
sq = lambda x: x * x
print(sq(5))

pair = lambda a, b: (a, b)
print(pair("x", 1))

apply = lambda f, x: f(x)
print(apply(lambda v: v + 1, 5))
