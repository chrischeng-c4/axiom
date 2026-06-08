# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "rmdir_nonempty_raises_oserror"
# subject = "os.rmdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.rmdir: os.rmdir on a directory that still contains a child raises OSError ('Directory not empty'); cleanup leaves no temp tree behind"""
import os
import tempfile

td = tempfile.mkdtemp()
sub = os.path.join(td, "child")
os.mkdir(sub)
raised = False
try:
    os.rmdir(td)
except OSError:
    raised = True
finally:
    os.rmdir(sub)
    os.rmdir(td)
assert raised, "rmdir on a non-empty directory should raise OSError"
print("rmdir_nonempty_raises_oserror OK")
