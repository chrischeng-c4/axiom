# decorator broad

# decorator with arg (factory)
def repeat(n):
    def deco(fn):
        def wrapper(*args, **kwargs):
            result = None
            for _ in range(n):
                result = fn(*args, **kwargs)
            return result
        return wrapper
    return deco

@repeat(3)
def say(msg):
    print(msg)
    return msg

result = say("hi")
print(result)

# stacked decorators
def double(fn):
    def wrapper(*args, **kwargs):
        r = fn(*args, **kwargs)
        return r * 2
    return wrapper

def plus_one(fn):
    def wrapper(*args, **kwargs):
        r = fn(*args, **kwargs)
        return r + 1
    return wrapper

@double
@plus_one
def raw(x):
    return x

print(raw(5))
print(raw(10))

# decorator preserves args/kwargs
def logged(fn):
    def wrapper(*args, **kwargs):
        return fn(*args, **kwargs)
    return wrapper

@logged
def full(a, b, c=10):
    return a + b + c

print(full(1, 2, 3))
print(full(1, 2, 100))
