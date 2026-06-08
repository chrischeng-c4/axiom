x = 1
y = 2

class C:
    pass

# At module scope locals() is equivalent to globals().
print(type(locals()).__name__)
print(locals()["x"])
print(locals()["y"])
