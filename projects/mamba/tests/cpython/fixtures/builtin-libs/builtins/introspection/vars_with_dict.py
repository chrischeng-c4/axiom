class P:
    def __init__(self):
        self.x = 1
        self.y = 2

p = P()
d = vars(p)
print(sorted(d.keys()))
print(d["x"])
print(d["y"])
