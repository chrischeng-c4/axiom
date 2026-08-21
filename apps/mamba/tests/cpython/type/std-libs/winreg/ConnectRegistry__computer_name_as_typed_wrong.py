# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "ConnectRegistry__computer_name_as_typed_wrong"
# subject = "winreg.ConnectRegistry(computer_name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.ConnectRegistry(computer_name: typed); call it with the wrong type.

typeshed contract: computer_name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import ConnectRegistry
try:
    ConnectRegistry(_W(), None)  # computer_name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
