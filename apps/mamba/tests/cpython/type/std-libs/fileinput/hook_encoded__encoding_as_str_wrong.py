# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "hook_encoded__encoding_as_str_wrong"
# subject = "fileinput.hook_encoded(encoding: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.hook_encoded(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fileinput import hook_encoded
try:
    hook_encoded(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
