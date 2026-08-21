# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "key_attribute_roundtrip"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: a constructed ZoneInfo carries its lookup key on .key and str() returns that key"""
import zoneinfo

for key in ["UTC", "America/New_York"]:
    zi = zoneinfo.ZoneInfo(key)
    assert zi.key == key, (zi.key, key)
    assert str(zi) == key, (str(zi), key)
    assert repr(zi) == "zoneinfo.ZoneInfo(key=%r)" % key, repr(zi)
print("key_attribute_roundtrip OK")
