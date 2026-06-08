# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "get_terminal_size_env_precedence"
# subject = "shutil.get_terminal_size"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.get_terminal_size: COLUMNS/LINES env vars take precedence over the real terminal; a malformed value is ignored (result stays >= 0); os.environ is saved and restored around the probe"""
import shutil
import os


def _with_env(changes, fn):
    """Run fn() with os.environ patched by `changes` (None = unset),
    restoring the previous state afterward."""
    saved = {k: os.environ.get(k) for k in changes}
    try:
        for k, v in changes.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        return fn()
    finally:
        for k, old in saved.items():
            if old is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = old


# COLUMNS/LINES env vars win over the real terminal.
size = _with_env({"COLUMNS": "777", "LINES": "888"}, shutil.get_terminal_size)
assert size.columns == 777, f"columns = {size.columns}"
assert size.lines == 888, f"lines = {size.lines}"
assert tuple(size) == (777, 888), f"tuple = {tuple(size)!r}"

# COLUMNS only set -> columns from env, lines from terminal/fallback (>= 0).
s2 = _with_env({"COLUMNS": "123", "LINES": None}, shutil.get_terminal_size)
assert s2.columns == 123, f"columns = {s2.columns}"
assert s2.lines >= 0, f"lines = {s2.lines}"

# A malformed env value is ignored; result is still sane (>= 0).
s3 = _with_env({"COLUMNS": "xxx", "LINES": "yyy"}, shutil.get_terminal_size)
assert s3.columns >= 0 and s3.lines >= 0, f"bad env -> {tuple(s3)!r}"

# With no env vars, the explicit fallback path stays sane (>= 0).
s4 = _with_env({"COLUMNS": None, "LINES": None},
               lambda: shutil.get_terminal_size(fallback=(10, 20)))
assert s4.columns >= 0 and s4.lines >= 0, f"fallback -> {tuple(s4)!r}"

print("get_terminal_size_env_precedence OK")
