# with-statement broad

# custom context manager
class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, *a):
        print("exit", self.name)
        return False

# basic with ... as
with CM("A") as a:
    print("body", a.name)

# nested
with CM("outer") as o:
    with CM("inner") as i:
        print("inside", o.name, i.name)

# multiple CMs on one line
with CM("X") as x, CM("Y") as y:
    print("both", x.name, y.name)

# __enter__ returns different value
class Box:
    def __init__(self, v):
        self.v = v
    def __enter__(self):
        return self.v
    def __exit__(self, *a):
        return False

with Box(42) as n:
    print(n)
    print(n + 1)

# exception propagation — __exit__ returns False
class Trace:
    def __enter__(self):
        print("t-enter")
        return self
    def __exit__(self, *a):
        print("t-exit")
        return False

try:
    with Trace():
        print("before")
        raise ValueError("boom")
except ValueError as e:
    print("caught", e)

# loop inside with
class Log:
    def __enter__(self):
        return []
    def __exit__(self, *a):
        return False

with Log() as buf:
    for i in range(3):
        buf.append(i)
    print(buf)
