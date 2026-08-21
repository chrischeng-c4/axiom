# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_context_manager_closes"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: the SpooledTemporaryFile context manager closes the file on exit; re-entering the closed spool raises ValueError"""
import tempfile

with tempfile.SpooledTemporaryFile(max_size=1) as cm:
    assert not cm.closed, "open inside with-block"
assert cm.closed, "closed after with-block"
_raised = False
try:
    with cm:
        pass
except ValueError:
    _raised = True
assert _raised, "reusing closed spool should raise ValueError"
print("spooled_context_manager_closes OK")
