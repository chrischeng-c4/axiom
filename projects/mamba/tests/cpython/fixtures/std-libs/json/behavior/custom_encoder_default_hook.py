# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "custom_encoder_default_hook"
# subject = "json.JSONEncoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_default.py"
# status = "filled"
# ///
"""json.JSONEncoder: a JSONEncoder subclass overriding default() serializes an otherwise-unsupported type (a set encoded as a sorted list)"""
import json


class SetEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, set):
            return sorted(obj)
        return super().default(obj)


encoded = json.dumps({3, 1, 2}, cls=SetEncoder)
assert json.loads(encoded) == [1, 2, 3], f"custom encoder = {json.loads(encoded)!r}"

print("custom_encoder_default_hook OK")
