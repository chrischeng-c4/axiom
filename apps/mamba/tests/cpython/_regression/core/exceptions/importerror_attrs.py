# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""ImportError name/path/msg attributes and keyword validation (CPython 3.12 oracle)."""

# Bare ImportError: name and path default to None.
exc = ImportError("test")
assert exc.name is None
assert exc.path is None
assert exc.msg == "test"

# name-only and path-only keywords.
exc = ImportError("test", name="somemodule")
assert exc.name == "somemodule"
assert exc.path is None

exc = ImportError("test", path="somepath")
assert exc.path == "somepath"
assert exc.name is None

# Both keywords together.
exc = ImportError("test", path="somepath", name="somename")
assert exc.name == "somename"
assert exc.path == "somepath"
assert exc.args == ("test",)
print("importerror_attrs: name/path/msg carried")


# An unknown keyword raises TypeError naming the bad keyword.
try:
    ImportError("test", invalid="keyword")
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "invalid" in str(e)
    print("bad_keyword: TypeError ->", str(e))


# Re-running __init__() with no args resets every attribute.
exc = ImportError("test", name="name", path="path")
assert exc.name == "name"
assert exc.path == "path"
exc.__init__()
assert exc.args == ()
assert exc.msg is None
assert exc.name is None
assert exc.path is None
print("reset: __init__() cleared name/path/msg")

print("importerror_attrs OK")
