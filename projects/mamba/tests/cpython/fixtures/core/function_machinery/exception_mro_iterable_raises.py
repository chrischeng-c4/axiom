# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `Exception.__mro__` raises 'object is not iterable' on
# mamba even though `issubclass` works. CPython yields the MRO tuple.
# See project_mamba_function_machinery_silent_divergences (#9).
"""Exception class __mro__ iteration (CPython 3.12 oracle)."""

# CPython: tuple of classes; mamba: error (or non-iterable).
try:
    names = [c.__name__ for c in ValueError.__mro__]
    print("mro_names:", names)
except (TypeError, AttributeError) as e:
    print(f"mro_names: {type(e).__name__}: {str(e)[:40]}")

# CPython: True (ValueError is a subclass of Exception). mamba: True.
print("subclass:", issubclass(ValueError, Exception))

# CPython: returns 'ValueError'; mamba: same.
print("name:", ValueError.__name__)

# CPython: tuple; mamba: NoneType.
print("mro_type:", type(ValueError.__mro__).__name__)
