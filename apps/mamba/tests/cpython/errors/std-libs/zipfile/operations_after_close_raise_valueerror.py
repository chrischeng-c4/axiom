# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "operations_after_close_raise_valueerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: after close(), each of read/open/testzip/writestr on the ZipFile raises ValueError"""
import zipfile
import io


def _expect_valueerror(fn):
    try:
        fn()
    except ValueError:
        return True
    return False


_buf = io.BytesIO()
_z = zipfile.ZipFile(_buf, "w")
_z.writestr("foo.txt", "data")
_z.close()

assert _expect_valueerror(lambda: _z.read("foo.txt")), "read after close -> ValueError"
assert _expect_valueerror(lambda: _z.open("foo.txt")), "open after close -> ValueError"
assert _expect_valueerror(lambda: _z.testzip()), "testzip after close -> ValueError"
assert _expect_valueerror(lambda: _z.writestr("b.txt", "x")), "writestr after close -> ValueError"

print("operations_after_close_raise_valueerror OK")
