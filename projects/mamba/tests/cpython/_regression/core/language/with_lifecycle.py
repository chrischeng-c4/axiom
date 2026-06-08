# Context manager lifecycle: __enter__/__exit__ runs on normal and
# exception paths; as-binding + method chaining

class Trace:
    def __init__(self, tag):
        self.tag = tag
    def __enter__(self):
        print(f"enter {self.tag}")
        return self.tag
    def __exit__(self, et, ev, tb):
        print(f"exit {self.tag}")
        return False

# Normal path
with Trace("A") as a:
    print("body", a)

# Exception propagates — __exit__ still fires
try:
    with Trace("B") as b:
        print("body", b)
        raise RuntimeError("boom")
except RuntimeError as e:
    print("caught:", e)

# as-binding used for method call
with Trace("C") as c:
    x = c.upper()
    print(x)

print("done")
