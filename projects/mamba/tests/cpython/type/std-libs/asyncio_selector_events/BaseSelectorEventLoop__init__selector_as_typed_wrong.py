# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_selector_events"
# dimension = "type"
# case = "BaseSelectorEventLoop__init__selector_as_typed_wrong"
# subject = "asyncio.selector_events.BaseSelectorEventLoop.__init__(selector: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/selector_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.selector_events.BaseSelectorEventLoop.__init__(selector: typed); call it with the wrong type.

typeshed contract: selector is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.selector_events import BaseSelectorEventLoop
try:
    BaseSelectorEventLoop(_W())  # selector: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
