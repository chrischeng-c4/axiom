# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""The assert statement and AssertionError semantics (CPython 3.12 oracle)."""

# A failing assert with a message raises AssertionError carrying that message.
try:
    assert False, "hello"
    raise AssertionError("expected the assert to fail")
except AssertionError as e:
    assert str(e) == "hello"
    print("assert_with_message: str=", str(e))


# A failing bare assert raises AssertionError with no message.
try:
    assert 1 == 2
    raise AssertionError("expected the assert to fail")
except AssertionError as e:
    assert str(e) == ""
    assert e.args == ()
    print("assert_bare: empty args")


# A truthy assert is a no-op.
ran = False
try:
    assert 1 == 1
    ran = True
except AssertionError:
    ran = False
assert ran is True
print("assert_pass: truthy assert is a no-op")


# The message expression may be any object, exposed via args[0].
try:
    assert 0, ("code", 42)
except AssertionError as e:
    assert e.args[0] == ("code", 42)
    print("assert_tuple_message: args0=", e.args[0])

print("assert_statement OK")
