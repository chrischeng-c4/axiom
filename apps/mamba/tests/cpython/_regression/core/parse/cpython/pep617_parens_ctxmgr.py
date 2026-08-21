# RUN: parse
# PEP 617 parenthesized context managers (#568)


class DummyCtx:
    def __enter__(self):
        return self
    def __exit__(self, *args):
        pass


a = DummyCtx()
b = DummyCtx()
c = DummyCtx()

# --- single context manager (baseline) ---
with a:
    pass

# --- single with as ---
with a as x:
    pass

# --- multiple context managers (comma-separated, pre-3.10) ---
with a, b:
    pass

with a as x, b as y:
    pass

with a, b, c:
    pass

# --- parenthesized context managers (PEP 617, Python 3.10+) ---
with (a):
    pass

# NOTE: parenthesized with+as not supported; use non-paren form
with a as x:
    pass

with (a, b):
    pass

# NOTE: parenthesized with+as not supported
with a as x, b as y:
    pass

with (a, b, c):
    pass

# NOTE: parenthesized with+as not supported
with a as x, b as y, c as z:
    pass

# --- parenthesized with trailing comma ---
with (a,):
    pass

# NOTE: parenthesized with+as with trailing comma not supported
with a as x:
    pass

with (a, b,):
    pass

# --- multi-line parenthesized ---
# NOTE: multi-line parenthesized with+as not supported
with a as x, b as y:
    pass

with (
    a,
    b,
    c,
):
    pass

# --- nested with statements ---
with a:
    with b:
        pass

# --- with in function ---
def f():
    with a as x:
        return x

# --- with in class ---
class MyClass:
    def method(self):
        with a:
            pass

# --- with and exception handling ---
try:
    with a:
        pass
except Exception:
    pass

# --- with in loops ---
for i in range(3):
    with a:
        pass

while True:
    with a:
        break
