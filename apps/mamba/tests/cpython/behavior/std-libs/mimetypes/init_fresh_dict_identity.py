# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "init_fresh_dict_identity"
# subject = "mimetypes.init"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.init: re-running init() yields fresh dict objects (new identity) with equal content for types_map/suffix_map/encodings_map/common_types"""
import mimetypes

mimetypes.init()
sm, em = mimetypes.suffix_map, mimetypes.encodings_map
tm, ct = mimetypes.types_map, mimetypes.common_types
mimetypes.init()
# Fresh identities ...
assert sm is not mimetypes.suffix_map, "suffix_map fresh object"
assert tm is not mimetypes.types_map, "types_map fresh object"
# ... but equal content.
assert sm == mimetypes.suffix_map, "suffix_map equal content"
assert em == mimetypes.encodings_map, "encodings_map equal content"
assert tm == mimetypes.types_map, "types_map equal content"
assert ct == mimetypes.common_types, "common_types equal content"
print("init_fresh_dict_identity OK")
