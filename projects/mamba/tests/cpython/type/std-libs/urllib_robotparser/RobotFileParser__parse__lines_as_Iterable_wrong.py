# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__parse__lines_as_Iterable_wrong"
# subject = "urllib.robotparser.RobotFileParser.parse(lines: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.parse(lines: Iterable); call it with the wrong type.

typeshed contract: lines is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.parse(_W())  # lines: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
