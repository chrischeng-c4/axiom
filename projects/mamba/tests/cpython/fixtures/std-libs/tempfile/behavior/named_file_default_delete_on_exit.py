# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_default_delete_on_exit"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: default delete=True removes the underlying file when the with-block exits, and reusing the closed file raises ValueError"""
import os
import tempfile

with tempfile.NamedTemporaryFile() as auto:
    assert os.path.exists(auto.name), "exists inside with-block"
    auto_name = auto.name
assert not os.path.exists(auto_name), "deleted after with-block"
_reraised = False
try:
    with auto:
        pass
except ValueError:
    _reraised = True
assert _reraised, "reusing the closed file should raise ValueError"
print("named_file_default_delete_on_exit OK")
