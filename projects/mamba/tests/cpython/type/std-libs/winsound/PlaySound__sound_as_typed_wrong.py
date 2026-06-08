# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winsound"
# dimension = "type"
# case = "PlaySound__sound_as_typed_wrong"
# subject = "winsound.PlaySound(sound: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sound"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winsound.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sound
# mamba-strict-type: TypeError
"""Type wall: winsound.PlaySound(sound: typed); call it with the wrong type.

typeshed contract: sound is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winsound import PlaySound
try:
    PlaySound(_W(), None)  # sound: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
