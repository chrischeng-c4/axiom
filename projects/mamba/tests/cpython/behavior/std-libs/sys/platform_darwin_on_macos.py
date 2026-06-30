# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "platform_darwin_on_macos"
# subject = "sys.platform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.platform: macOS reports CPython's historical darwin platform tag."""
import platform
import sys

if platform.system() == "Darwin":
    assert sys.platform == "darwin", sys.platform
else:
    assert isinstance(sys.platform, str) and len(sys.platform) > 0
print("platform_darwin_on_macos OK")
