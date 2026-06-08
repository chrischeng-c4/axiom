# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Function `__qualname__` and `__module__` were both returning None for
# top-level defs. CPython returns the source name for `__qualname__`
# (qualname == name for top-level functions) and `'__main__'` for
# `__module__`. The fix extends `mb_getattr`'s existing __name__ branch
# to share the FUNC_NAMES registry with __qualname__, and gates the
# `__module__` fallback behind a registry-presence check so plain
# integers don't accidentally claim a module.

def foo(): pass

print(foo.__name__)        # foo
print(foo.__qualname__)    # foo
print(foo.__module__)      # __main__

# Multiple top-level defs all register correctly.
def bar(): return 1
def baz(): return 2
print(bar.__qualname__)    # bar
print(baz.__module__)      # __main__
