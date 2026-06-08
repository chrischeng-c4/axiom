# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "tlsversion_ordering_and_sentinels"
# subject = "ssl.TLSVersion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.TLSVersion: concrete TLSVersion members order as integers (TLSv1_2 < TLSv1_3, TLSv1_3 == 772) while MAXIMUM_SUPPORTED / MINIMUM_SUPPORTED are the -1 / -2 sentinels"""
import ssl

assert ssl.TLSVersion.TLSv1_2 < ssl.TLSVersion.TLSv1_3, "TLSv1_2 < TLSv1_3"
assert int(ssl.TLSVersion.TLSv1_3) == 772, f"TLSv1_3 = {int(ssl.TLSVersion.TLSv1_3)}"
assert int(ssl.TLSVersion.MAXIMUM_SUPPORTED) == -1, "MAXIMUM_SUPPORTED sentinel"
assert int(ssl.TLSVersion.MINIMUM_SUPPORTED) == -2, "MINIMUM_SUPPORTED sentinel"

print("tlsversion_ordering_and_sentinels OK")
