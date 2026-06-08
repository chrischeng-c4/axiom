# nested function / closure broad

# nested fn call
def outer():
    def inner():
        return "inner-result"
    return inner()

print(outer())

# fn as argument
def apply_twice(f, x):
    return f(f(x))

def inc(n):
    return n + 1

print(apply_twice(inc, 5))
print(apply_twice(lambda x: x * 2, 3))

# fn returned and called chain
def return_fn():
    def my_fn(x):
        return x + 1000
    return my_fn

r = return_fn()
print(r(5))
print(r(15))

# decorator style inline
def wrap(f):
    def wrapped(x):
        return f(x) + 1
    return wrapped

@wrap
def raw(x):
    return x * 2

print(raw(10))
print(raw(5))

# nested dict from fn
def make_point(x, y):
    return {"x": x, "y": y}

p = make_point(3, 4)
print(p["x"], p["y"])

# fn returning string
def greeter(name):
    return "hello, " + name

print(greeter("alice"))
print(greeter("bob"))

# deeply nested
def level1():
    def level2():
        def level3():
            return "deepest"
        return level3()
    return level2()

print(level1())

# fn returning list
def make_list(n):
    return [i for i in range(n)]

print(make_list(5))
print(make_list(3))
