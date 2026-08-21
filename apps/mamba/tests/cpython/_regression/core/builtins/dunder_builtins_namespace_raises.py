# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `__builtins__` namespace is near-empty on mamba;
# `hasattr(__builtins__, 'print')` is False even though `print` is
# callable globally. CPython exposes 150+ names. See
# project_mamba_module_exec_del_silent_divergences (#2).
"""__builtins__ namespace surface (CPython 3.12 oracle)."""

# CPython: True for both; mamba: False.
print("has_print:", hasattr(__builtins__, "print"))
print("has_len:",   hasattr(__builtins__, "len"))
print("has_range:", hasattr(__builtins__, "range"))
print("has_open:",  hasattr(__builtins__, "open"))

# CPython: True; mamba: True (the callables themselves still work).
print("callable_print:", callable(print))
print("callable_len:",   callable(len))

# CPython: many; mamba: few.
try:
    n = len(dir(__builtins__))
    print("dir_len_gt_50:", n > 50)
except TypeError as e:
    print(f"dir_len: TypeError: {str(e)[:40]}")
