# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `hash([1, 2])` silently returns an int on mamba;
# CPython raises TypeError "unhashable type: 'list'". See
# project_mamba_function_machinery_silent_divergences (#7).
"""builtin hash() unhashable contract (CPython 3.12 oracle)."""

# CPython raises TypeError on each of these; mamba silently hashes.
for label, value in (
    ("list",      [1, 2]),
    ("dict",      {"a": 1}),
    ("set",       {1, 2}),
    ("nested",    [[1], [2]]),
):
    try:
        h = hash(value)
        print(f"{label}: no_raise: {type(h).__name__}")
    except TypeError as e:
        print(f"{label}: TypeError: {str(e)[:40]}")
