# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "excepthook_formats_active_exception"
# subject = "sys.__excepthook__"
# kind = "semantic"
# xfail = "mamba exc_info / traceback formatting incomplete (repo-memory project_mamba_traceback_format_exc_stub)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__excepthook__: __excepthook__(*exc_info()) for an active ValueError(42) writes a traceback to stderr ending in 'ValueError: 42\\n'"""
import io
import sys
from contextlib import redirect_stderr

try:
    raise ValueError(42)
except ValueError:
    err = io.StringIO()
    with redirect_stderr(err):
        sys.__excepthook__(*sys.exc_info())
assert err.getvalue().endswith("ValueError: 42\n"), \
    f"excepthook tail = {err.getvalue()[-40:]!r}"
print("excepthook_formats_active_exception OK")
