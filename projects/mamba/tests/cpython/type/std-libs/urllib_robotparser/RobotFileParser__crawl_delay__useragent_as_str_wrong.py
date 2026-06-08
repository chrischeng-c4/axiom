# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__crawl_delay__useragent_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.crawl_delay(useragent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.crawl_delay(useragent: str); call it with the wrong type.

typeshed contract: useragent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.crawl_delay(12345)  # useragent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
