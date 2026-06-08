# decorator with args deep broad

# simple factory: repeat N times and collect
def repeat(n):
    def wrap(fn):
        def inner(*args):
            r = []
            for _ in range(n):
                r.append(fn(*args))
            return r
        return inner
    return wrap

@repeat(3)
def greet(name):
    return "hi " + name

print(greet("Alice"))

@repeat(5)
def get_one():
    return 1

print(get_one())

# different decorator for each fn
def double(fn):
    def inner(*args):
        return fn(*args) * 2
    return inner

@double
def nine():
    return 9

print(nine())

@double
def sum_ab(a, b):
    return a + b

print(sum_ab(1, 2))

# stacked: @outer @inner
def add100(fn):
    def inner(*args):
        return fn(*args) + 100
    return inner

@double
@add100
def val():
    return 5
# first add100(val)=105, then double(105)=210
print(val())

# decorator preserving args
def logged(fn):
    def inner(*args):
        r = fn(*args)
        return r
    return inner

@logged
def mul(a, b):
    return a * b

print(mul(3, 4))
print(mul(5, 6))

# decorator with factory args 2
def prefix(p):
    def wrap(fn):
        def inner(*args):
            return p + str(fn(*args))
        return inner
    return wrap

@prefix("RES: ")
def compute():
    return 42

print(compute())
