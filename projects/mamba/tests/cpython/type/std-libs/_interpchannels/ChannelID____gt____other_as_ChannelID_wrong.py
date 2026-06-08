# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "ChannelID____gt____other_as_ChannelID_wrong"
# subject = "_interpchannels.ChannelID.__gt__(other: ChannelID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.ChannelID.__gt__(other: ChannelID); call it with the wrong type.

typeshed contract: other is ChannelID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import ChannelID
obj = object.__new__(ChannelID)
try:
    obj.__gt__(_W())  # other: ChannelID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
