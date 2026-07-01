# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "IndentedHelpFormatter__format_heading__heading_as_str_wrong"
# subject = "optparse.IndentedHelpFormatter.format_heading(heading: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.IndentedHelpFormatter.format_heading(heading: str); call it with the wrong type.

typeshed contract: heading is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import IndentedHelpFormatter
obj = object.__new__(IndentedHelpFormatter)
try:
    obj.format_heading(12345)  # heading: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
