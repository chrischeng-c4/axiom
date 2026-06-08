# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `del lst[a:b]` is a no-op on mamba; CPython removes the
# slice. Single-index `del lst[i]` works on both. See
# project_mamba_module_exec_del_silent_divergences (#5).
"""del with slice (CPython 3.12 oracle)."""

lst = [1, 2, 3, 4, 5]
del lst[1:4]
# CPython: [1, 5]; mamba: [1, 2, 3, 4, 5].
print("after_slice:", lst)

# CPython: True; mamba: False.
print("len_changed:", len(lst) == 2)

# Single-index del works on both.
lst2 = [1, 2, 3, 4, 5]
del lst2[2]
print("after_index:", lst2)

# Slice del on a deeper structure to catch any partial implementation.
nested = [[1, 2], [3, 4], [5, 6]]
del nested[:2]
print("nested:", nested)
