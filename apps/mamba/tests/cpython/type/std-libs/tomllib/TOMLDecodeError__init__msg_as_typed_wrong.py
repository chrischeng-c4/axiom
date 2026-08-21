# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "type"
# case = "TOMLDecodeError__init__msg_as_typed_wrong"
# subject = "tomllib.TOMLDecodeError.__init__(msg: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tomllib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg
# mamba-strict-type: TypeError
"""Type wall: tomllib.TOMLDecodeError.__init__(msg: typed); call it with the wrong type.

typeshed contract: msg is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tomllib import TOMLDecodeError
try:
    TOMLDecodeError(_W())  # msg: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
