# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `e.__traceback__` is None on mamba after a caught
# exception; CPython exposes a traceback object. See
# project_mamba_module_exec_del_silent_divergences (#1).
"""Exception __traceback__ attribute (CPython 3.12 oracle)."""


def boom() -> None:
    raise ValueError("inside boom")


try:
    boom()
except ValueError as e:
    tb = e.__traceback__
    # CPython: 'traceback'; mamba: 'NoneType'.
    print("tb_type:", type(tb).__name__)

    # CPython: not None; mamba: None.
    print("tb_is_none:", tb is None)

    # Accessing frame attributes raises on None.
    try:
        line = tb.tb_lineno
        print("tb_lineno_type:", type(line).__name__)
    except AttributeError as ee:
        print(f"tb_lineno: AttributeError: {str(ee)[:40]}")
