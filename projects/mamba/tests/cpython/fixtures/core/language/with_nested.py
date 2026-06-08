# Nested / multi-binding with statements

class Tag:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print(f"<{self.name}>")
        return self.name
    def __exit__(self, et, ev, tb):
        print(f"</{self.name}>")
        return False

# Multi-binding `with ... as a, ... as b:` form
with Tag("outer") as a, Tag("inner") as b:
    print(a, b)

# Nested with-blocks
with Tag("x") as a:
    with Tag("y") as b:
        print("body", a, b)
