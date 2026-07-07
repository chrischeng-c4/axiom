# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""f_back frames expose the caller's locals snapshot (CPython 3.12 oracle)."""

import inspect


def outer():
    marker = 123

    def inner():
        frame = inspect.currentframe()
        back = frame.f_back
        assert back.f_locals.get("marker") == 123

    inner()


outer()
print("frame_back_locals_snapshot OK")
