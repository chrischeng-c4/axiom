# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "type"
# case = "RadioButtonGroup__init__dlg_as_Dialog_wrong"
# subject = "msilib.RadioButtonGroup.__init__(dlg: Dialog)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msilib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msilib.RadioButtonGroup.__init__(dlg: Dialog); call it with the wrong type.

typeshed contract: dlg is Dialog. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msilib import RadioButtonGroup
try:
    RadioButtonGroup(_W(), "", "")  # dlg: Dialog <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
