# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator with try/finally around yields — for-loop exhaustion path.
# Complements generators/lifecycle.py (which covers close() triggering
# the finally). Here the finally runs naturally when the generator is
# drained, and nested try/finally preserves inner-then-outer ordering.

def gen_with_finally():
    try:
        yield 1
        yield 2
        yield 3
    finally:
        print("cleanup")

for v in gen_with_finally():
    print(v)

# try/except inside a generator — throw() routed to the except arm.
def gen_with_except():
    try:
        yield 1
        yield 2
    except ValueError:
        yield "caught"

g = gen_with_except()
print(next(g))
print(g.throw(ValueError))

# Nested try/finally — inner finally runs before outer on exhaustion.
def nested():
    try:
        try:
            yield 1
            yield 2
        finally:
            print("inner")
    finally:
        print("outer")

for v in nested():
    print(v)
