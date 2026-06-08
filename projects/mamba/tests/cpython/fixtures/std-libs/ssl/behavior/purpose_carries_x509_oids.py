# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "purpose_carries_x509_oids"
# subject = "ssl.Purpose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.Purpose: Purpose members carry the well-known X.509 EKU OIDs: SERVER_AUTH is nid 129 / serverAuth / 1.3.6.1.5.5.7.3.1 and CLIENT_AUTH is nid 130 / clientAuth / 1.3.6.1.5.5.7.3.2, and the two are distinct"""
import ssl

_sa = ssl.Purpose.SERVER_AUTH
assert _sa.nid == 129, f"SERVER_AUTH nid = {_sa.nid}"
assert _sa.shortname == "serverAuth", f"SERVER_AUTH shortname = {_sa.shortname}"
assert _sa.oid == "1.3.6.1.5.5.7.3.1", f"SERVER_AUTH oid = {_sa.oid}"

_ca = ssl.Purpose.CLIENT_AUTH
assert _ca.nid == 130, f"CLIENT_AUTH nid = {_ca.nid}"
assert _ca.shortname == "clientAuth", f"CLIENT_AUTH shortname = {_ca.shortname}"
assert _ca.oid == "1.3.6.1.5.5.7.3.2", f"CLIENT_AUTH oid = {_ca.oid}"

assert _sa != _ca, "the two purposes are distinct"

print("purpose_carries_x509_oids OK")
