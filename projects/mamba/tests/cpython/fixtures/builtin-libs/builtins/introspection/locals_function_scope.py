def f(a, b):
    c = a + b
    return locals()

r = f(10, 20)
print(type(r).__name__)
print(sorted(r.keys()))
print(r["a"])
print(r["b"])
print(r["c"])
