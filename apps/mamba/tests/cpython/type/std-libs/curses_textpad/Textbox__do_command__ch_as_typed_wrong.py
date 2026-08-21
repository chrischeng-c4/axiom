# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_textpad"
# dimension = "type"
# case = "Textbox__do_command__ch_as_typed_wrong"
# subject = "curses.textpad.Textbox.do_command(ch: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/textpad.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.textpad.Textbox.do_command(ch: typed); call it with the wrong type.

typeshed contract: ch is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.textpad import Textbox
obj = object.__new__(Textbox)
try:
    obj.do_command(_W())  # ch: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
