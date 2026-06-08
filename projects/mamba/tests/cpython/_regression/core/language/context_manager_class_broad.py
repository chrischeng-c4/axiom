# context manager broad

# class-based
class Resource:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print(f"open {self.name}")
        return self
    def __exit__(self, et, ev, tb):
        print(f"close {self.name}")

with Resource("A") as r:
    print(f"using {r.name}")

# nested
with Resource("B") as b:
    with Resource("C") as c:
        print(f"both {b.name} {c.name}")

# multiple on single line
with Resource("D") as d, Resource("E") as e:
    print(f"multi {d.name} {e.name}")

# __exit__ not swallowing exception
class Silent:
    def __enter__(self):
        return self
    def __exit__(self, et, ev, tb):
        print("cleanup")

try:
    with Silent():
        raise ValueError("oops")
except ValueError as e:
    print(f"caught: {e}")

# __enter__ returns something else
class Fake:
    def __enter__(self):
        return "the value"
    def __exit__(self, et, ev, tb):
        pass

with Fake() as val:
    print(val)
    print(type(val).__name__)

# class with state
class Counter:
    def __init__(self):
        self.n = 0
    def __enter__(self):
        self.n += 1
        return self.n
    def __exit__(self, *args):
        pass

c = Counter()
with c as v1:
    print("first", v1)
with c as v2:
    print("second", v2)
print("final", c.n)
