# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/context_manager: with-protocol surface probes (CPython 3.12 oracle).

Shape of the context-manager protocol that the `with` statement drives:
a context manager exposes ``__enter__`` and ``__exit__``; the value bound
after ``as`` is whatever ``__enter__`` returns.
"""


class CM:
    def __enter__(self):
        return "entered"

    def __exit__(self, exc_type, exc, tb):
        return False


# A user context manager exposes both protocol methods.
cm = CM()
assert hasattr(cm, "__enter__")
assert hasattr(cm, "__exit__")
assert callable(cm.__enter__)
assert callable(cm.__exit__)

# The `with` statement binds `as` to the __enter__ return value, not the
# context-manager object itself.
with CM() as bound:
    assert bound == "entered"

# __exit__ takes exactly three positional args (exc_type, exc, tb).
import inspect
params = list(inspect.signature(CM.__exit__).parameters)
assert params == ["self", "exc_type", "exc", "tb"], params

# A self-returning context manager binds `as` to the same object.
class SelfCM:
    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False

obj = SelfCM()
with obj as same:
    assert same is obj

print("surface OK")
