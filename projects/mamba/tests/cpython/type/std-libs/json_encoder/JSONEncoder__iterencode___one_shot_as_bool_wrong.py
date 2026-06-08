# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "JSONEncoder__iterencode___one_shot_as_bool_wrong"
# subject = "json.encoder.JSONEncoder.iterencode(_one_shot: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _one_shot"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _one_shot
# mamba-strict-type: TypeError
"""Type wall: json.encoder.JSONEncoder.iterencode(_one_shot: bool); call it with the wrong type.

typeshed contract: _one_shot is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import JSONEncoder
obj = object.__new__(JSONEncoder)
try:
    obj.iterencode(None, "not_a_bool")  # _one_shot: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
