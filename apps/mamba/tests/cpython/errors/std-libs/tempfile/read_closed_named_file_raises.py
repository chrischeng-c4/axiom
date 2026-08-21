# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "read_closed_named_file_raises"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: reading a NamedTemporaryFile after it has been closed raises ValueError (the open w+b file reads fine; only the closed handle raises)"""
import os
import tempfile

f = tempfile.NamedTemporaryFile(delete=False)
f.write(b"data")
path = f.name
f.close()
_raised = False
try:
    f.read()
except ValueError:
    _raised = True
finally:
    try:
        os.unlink(path)
    except OSError:
        pass
assert _raised, "read_closed_named_file_raises: expected ValueError"
print("read_closed_named_file_raises OK")
