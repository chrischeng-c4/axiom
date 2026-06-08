# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "str_comment_raises_typeerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: assigning a str (not bytes) to ZipFile.comment raises TypeError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _raised = False
    try:
        _z.comment = "not bytes"
    except TypeError:
        _raised = True
    assert _raised, "str comment -> TypeError"

print("str_comment_raises_typeerror OK")
