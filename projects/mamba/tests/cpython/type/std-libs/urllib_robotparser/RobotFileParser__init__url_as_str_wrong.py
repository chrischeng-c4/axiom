# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__init__url_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.__init__(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.__init__(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
try:
    RobotFileParser(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
