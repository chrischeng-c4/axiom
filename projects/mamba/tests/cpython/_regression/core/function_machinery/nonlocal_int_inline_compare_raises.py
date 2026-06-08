# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: nonlocal-mutated int has boxing-equality bug — inline
# `outer() == 99` returns False on mamba, but assigning to a local
# first works. See project_mamba_function_machinery_silent_divergences
# (#10) and project_mamba_boxed_accumulator_int_equality_bug.
"""nonlocal int equality after inner mutation (CPython 3.12 oracle)."""


def outer() -> int:
    x = 1

    def inner() -> None:
        nonlocal x
        x = 99
    inner()
    return x


# CPython: 99; mamba: 99 too (the value itself is right).
print("value:", outer())

# CPython: True. Mamba: False due to boxing-eq divergence.
print("inline_eq:", outer() == 99)

# CPython: True. Mamba: True (workaround via local binding).
v = outer()
print("local_eq:", v == 99)
