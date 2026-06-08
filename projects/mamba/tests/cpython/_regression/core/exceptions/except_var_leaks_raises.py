# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `except X as e` keeps `e` bound after the block on
# mamba; CPython unbinds it (NameError on later access). See
# project_mamba_module_exec_del_silent_divergences (#6).
"""Except-clause variable cleanup (CPython 3.12 oracle)."""

try:
    raise ValueError("scoped")
except ValueError as e:
    # CPython: ok inside block.
    print("inside:", str(e)[:20])

# CPython: NameError "name 'e' is not defined". Mamba: prints 'scoped'.
try:
    print("after:", str(e)[:20])
except NameError as ne:
    print(f"after: NameError: {str(ne)[:40]}")
