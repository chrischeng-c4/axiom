def loud(fn):
    def wrapped(*args, **kwargs):
        print(f"calling {fn.__name__}")
        result = fn(*args, **kwargs)
        print(f"done {fn.__name__}")
        return result
    return wrapped

@loud
def greet(name):
    return f"hi {name}"

print(greet("alice"))

@loud
def add(a, b):
    return a + b

print(add(3, 4))

# stacked
def upper_result(fn):
    def wrapped(*args, **kwargs):
        return str(fn(*args, **kwargs)).upper()
    return wrapped

def repeat(fn):
    def wrapped(*args, **kwargs):
        result = fn(*args, **kwargs)
        return result + result
    return wrapped

@upper_result
@repeat
def name():
    return "alice"

print(name())

# parametrized
def repeat_n(n):
    def deco(fn):
        def wrapped(*args, **kwargs):
            results = []
            for _ in range(n):
                results.append(fn(*args, **kwargs))
            return results
        return wrapped
    return deco

@repeat_n(3)
def hello():
    return "hi"

print(hello())
