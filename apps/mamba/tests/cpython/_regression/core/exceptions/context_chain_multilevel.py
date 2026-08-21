# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Multi-level implicit __context__ across try/except/finally (CPython 3.12 oracle)."""

# Exception in finally chains onto the exception from the try body.
try:
    try:
        te = TypeError(1)
        raise te
    finally:
        ve = ValueError(2)
        raise ve
except Exception as e:
    exc = e
assert exc is ve
assert exc.__context__ is te
print("try_finally: ve.__context__ is te")


# Three-level chain: finally over except over try.
try:
    try:
        te = TypeError(1)
        raise te
    except Exception:
        ve = ValueError(2)
        raise ve
    finally:
        oe = OSError(3)
        raise oe
except Exception as e:
    exc = e
assert exc is oe
assert exc.__context__ is ve
assert exc.__context__.__context__ is te
print("except_finally: oe -> ve -> te chain")


# Exception in else chains onto an exception raised in finally.
try:
    try:
        pass
    except Exception:
        pass
    else:
        ve = ValueError(1)
        raise ve
    finally:
        oe = OSError(2)
        raise oe
except Exception as e:
    exc = e
assert exc is oe
assert exc.__context__ is ve
print("else_finally: oe.__context__ is ve")


# Re-raising an already-caught exception relinks the chain without a cycle.
class A(Exception):
    pass


class B(Exception):
    pass


class C(Exception):
    pass


try:
    try:
        raise A
    except A as a_:
        a = a_
        try:
            raise B
        except B as b_:
            b = b_
            try:
                raise C
            except C as c_:
                c = c_
                assert b.__context__ is a
                assert c.__context__ is b
                raise a
except A as e:
    exc = e
assert exc is a
assert a.__context__ is c
assert c.__context__ is b
assert b.__context__ is None
print("no_cycle: reraise relinks a -> c -> b -> None")

print("context_chain_multilevel OK")
