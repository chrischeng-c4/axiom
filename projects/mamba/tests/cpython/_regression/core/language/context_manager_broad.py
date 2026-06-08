class Resource:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print(f"enter {self.name}")
        return self
    def __exit__(self, exc_type, exc_val, exc_tb):
        print(f"exit {self.name}")
        return False

with Resource("r1"):
    print("body")

with Resource("r2") as r:
    print(f"using {r.name}")

with Resource("outer") as outer:
    with Resource("inner") as inner:
        print("double body")

with Resource("a") as a, Resource("b") as b:
    print(f"multi: {a.name}, {b.name}")

with Resource("s1") as r1:
    print(f"s1={r1.name}")
with Resource("s2") as r2:
    print(f"s2={r2.name}")

for i in range(3):
    with Resource(f"loop-{i}") as r:
        print(f"iter {i}: {r.name}")
