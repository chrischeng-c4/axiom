# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "create__config_as_typed_wrong"
# subject = "_interpreters.create(config: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.create(config: typed); call it with the wrong type.

typeshed contract: config is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import create
try:
    create(_W())  # config: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
