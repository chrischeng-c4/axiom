# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_dnd"
# dimension = "type"
# case = "dnd_start__source_as__DndSource_wrong"
# subject = "tkinter.dnd.dnd_start(source: _DndSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/dnd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.dnd.dnd_start(source: _DndSource); call it with the wrong type.

typeshed contract: source is _DndSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.dnd import dnd_start
try:
    dnd_start(_W(), None)  # source: _DndSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
