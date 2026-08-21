# Operational AssertionPass seed for atexit register/unregister surface.
# Surface: `atexit.register(func, *args, **kwargs)` records `func` to
# run at interpreter shutdown and returns `func` itself (so the call
# can be used as a decorator). `atexit.unregister(func)` removes every
# registered callback that compares equal to `func` and returns None.
# Calling `unregister` on a function that is not registered is a
# silent no-op. Both functions accept regular def-functions, lambdas,
# and partially-applied callables; registration of the same function
# multiple times is allowed (unregister removes all instances).
import atexit
_ledger: list[int] = []

# register returns the function so it can act as a decorator
def cleanup() -> None:
    pass
result = atexit.register(cleanup)
assert result is cleanup; _ledger.append(1)

# unregister returns None
assert atexit.unregister(cleanup) is None; _ledger.append(1)

# Multi-register / multi-unregister cycle is safe
def f1() -> None:
    pass
def f2() -> None:
    pass
atexit.register(f1)
atexit.register(f2)
atexit.unregister(f1)
atexit.unregister(f2)
_ledger.append(1)

# Lambda registration round-trip
ll = lambda: None
r = atexit.register(ll)
assert r is ll; _ledger.append(1)
atexit.unregister(ll)

# Register with positional + keyword arguments
def with_args(x: int, y: int, z: int = 3) -> tuple[int, int, int]:
    return (x, y, z)
r2 = atexit.register(with_args, 1, 2, z=4)
assert r2 is with_args; _ledger.append(1)
atexit.unregister(with_args)

# Unregistering a function that was never registered is a silent no-op
def never_registered() -> None:
    pass
atexit.unregister(never_registered)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_atexit_register_unregister_ops {sum(_ledger)} asserts")
