# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: atexit — register / unregister API contract.
#   register(fn) returns fn back (CPython 3.0+ decorator-friendly contract),
#   unregister(fn) returns None.
# The actual callback firing at interpreter shutdown is NOT YET wired on
# mamba — registered callbacks do not run when the program exits. This seed
# therefore asserts the API shape (return values, repeated register /
# unregister idempotency) rather than execution of the callbacks at shutdown;
# tracked separately.
import atexit

_ledger: list[int] = []

def _cb1():
    pass

def _cb2():
    pass

# register(fn) returns fn so it can be used as a decorator
_r = atexit.register(_cb1)
assert _r is _cb1, "atexit.register(fn) returns fn (decorator contract)"
_ledger.append(1)

# register works for a second distinct callback too
_r2 = atexit.register(_cb2)
assert _r2 is _cb2, "atexit.register(fn) returns the same fn for a second registration"
_ledger.append(1)

# Re-registering the same callback returns the same function reference
_r3 = atexit.register(_cb1)
assert _r3 is _cb1, "re-registering the same callback returns the same fn"
_ledger.append(1)

# unregister returns None
_u = atexit.unregister(_cb1)
assert _u is None, "atexit.unregister(fn) returns None"
_ledger.append(1)

# unregister is idempotent (calling on an already-removed callback is fine)
_u2 = atexit.unregister(_cb1)
assert _u2 is None, "atexit.unregister is idempotent — second call returns None"
_ledger.append(1)

# unregister on a never-registered callback also returns None (does not raise)
def _never_registered():
    pass

_u3 = atexit.unregister(_never_registered)
assert _u3 is None, "atexit.unregister of an unregistered callback returns None"
_ledger.append(1)

# Cleanup: drop the lingering _cb2 registration
atexit.unregister(_cb2)

print(f"MAMBA_ASSERTION_PASS: test_atexit {sum(_ledger)} asserts")
