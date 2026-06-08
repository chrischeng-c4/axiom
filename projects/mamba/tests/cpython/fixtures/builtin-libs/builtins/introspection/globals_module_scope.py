x = 42
y = "hello"
def f(): return 1
g = globals()
print(type(g).__name__)
print("x" in g)
print(g["x"])
print(g["y"])
print(g["f"]())
