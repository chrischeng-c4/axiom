# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `*args` binds to a LIST on mamba; CPython binds it to a
# tuple. Equality with `(1, 2)` therefore returns False on mamba. See
# project_mamba_function_machinery_silent_divergences (#5).
"""*args binding type contract (CPython 3.12 oracle)."""


def f(*args):
    return args


a = f(1, 2)

# CPython: 'tuple'; mamba: 'list'.
print("type:", type(a).__name__)

# CPython: True; mamba: False.
print("eq_tuple:", a == (1, 2))

# CPython: True; mamba: True (both runtimes preserve elements).
print("len:", len(a))

# CPython: hash(...) works; mamba: TypeError because list is unhashable.
try:
    h = hash(a)
    print("hash: ok")
except TypeError as e:
    print(f"hash: TypeError: {str(e)[:40]}")
