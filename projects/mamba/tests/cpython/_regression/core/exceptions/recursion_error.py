# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""RecursionError from unbounded recursion (CPython 3.12 oracle)."""

import sys

# Lower the limit so the test is fast and deterministic.
old_limit = sys.getrecursionlimit()
sys.setrecursionlimit(200)
try:
    # Infinite self-recursion raises RecursionError with a standard message.
    def recurse():
        return recurse()

    try:
        recurse()
        raise AssertionError("expected RecursionError")
    except RecursionError as e:
        assert "maximum recursion depth exceeded" in str(e)
        assert isinstance(e, RuntimeError)
        print("direct: RecursionError is a RuntimeError")

    # A function that catches its own RecursionError can recover and return.
    def collect():
        try:
            return collect()
        except RecursionError as e:
            return e

    result = collect()
    assert isinstance(result, RecursionError)
    print("recovered: caught RecursionError and returned normally")
finally:
    sys.setrecursionlimit(old_limit)

print("recursion_error OK")
