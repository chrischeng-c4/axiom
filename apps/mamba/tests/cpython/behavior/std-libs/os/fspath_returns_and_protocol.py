# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fspath_returns_and_protocol"
# subject = "os.fspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fspath: os.fspath returns str/bytes unchanged and honors the __fspath__ protocol on a PathLike object"""
import os

# os.fspath returns str/bytes unchanged.
assert os.fspath("/tmp/x") == "/tmp/x", "fspath str"
assert os.fspath(b"/tmp/x") == b"/tmp/x", "fspath bytes"


# os.fspath honors the __fspath__ protocol on PathLike objects.
class P:
    def __fspath__(self):
        return "/tmp/from_protocol"


assert os.fspath(P()) == "/tmp/from_protocol", "fspath __fspath__"
print("fspath_returns_and_protocol OK")
