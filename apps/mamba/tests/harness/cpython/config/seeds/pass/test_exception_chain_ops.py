# Operational AssertionPass seed for `raise X from Y` explicit
# exception chaining (PEP 3134) and user-defined Exception subclasses.
# Surface: a custom subclass of Exception can be raised and caught
# by name; `raise NEW from PREV` sets `__cause__` to PREV so the
# original exception is reachable from the new one's chain.
_ledger: list[int] = []

class MyError(Exception):
    pass

# A user-defined subclass can be raised and caught
caught = ""
try:
    raise MyError("custom")
except MyError as e:
    caught = type(e).__name__
assert caught == "MyError"; _ledger.append(1)

# Subclass-of-Exception still matches the broader Exception clause
fallback_caught = ""
try:
    raise MyError("custom2")
except Exception as e:
    fallback_caught = type(e).__name__
assert fallback_caught == "MyError"; _ledger.append(1)

# raise-from sets __cause__ to the inner exception
chained_type = ""
cause_type = ""
try:
    try:
        raise ValueError("inner")
    except ValueError as e:
        raise RuntimeError("outer") from e
except RuntimeError as e:
    chained_type = type(e).__name__
    if e.__cause__ is not None:
        cause_type = type(e.__cause__).__name__
assert chained_type == "RuntimeError"; _ledger.append(1)
assert cause_type == "ValueError"; _ledger.append(1)

# A custom exception's message is preserved in str(e)
msg = ""
try:
    raise MyError("with a message")
except MyError as e:
    msg = str(e)
assert msg == "with a message"; _ledger.append(1)

# isinstance against the user class and the parent class
inst = MyError("inst")
assert isinstance(inst, MyError); _ledger.append(1)
assert isinstance(inst, Exception); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_exception_chain_ops {sum(_ledger)} asserts")
