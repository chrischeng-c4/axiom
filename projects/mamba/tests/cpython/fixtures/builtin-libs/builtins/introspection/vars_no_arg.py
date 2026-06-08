def f():
    a = 7
    return vars()

r = f()
print(type(r).__name__)
print(r["a"])
