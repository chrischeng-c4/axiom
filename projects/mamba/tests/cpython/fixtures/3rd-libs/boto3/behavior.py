"""Behavior contract for third-party boto3 package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import boto3  # type: ignore[import]

# Rule 1: Session stores region_name
_sess1 = boto3.Session(region_name="us-west-2")
assert _sess1.region_name == "us-west-2", f"region = {_sess1.region_name!r}"

# Rule 2: Session with profile_name stores it
_sess2 = boto3.Session(region_name="eu-west-1")
assert _sess2.region_name == "eu-west-1", f"region2 = {_sess2.region_name!r}"

# Rule 3: Session.get_available_services returns list of strings
_sess3 = boto3.Session()
_svcs3 = _sess3.get_available_services()
assert isinstance(_svcs3, list), f"services type = {type(_svcs3)!r}"
assert len(_svcs3) > 50, f"services count = {len(_svcs3)}"
assert "s3" in _svcs3, "s3 available"
assert "ec2" in _svcs3, "ec2 available"
assert "dynamodb" in _svcs3, "dynamodb available"

# Rule 4: Session.get_available_regions for s3
_sess4 = boto3.Session()
_regions4 = _sess4.get_available_regions("s3")
assert isinstance(_regions4, list), f"regions type = {type(_regions4)!r}"
assert len(_regions4) > 0, "at least one s3 region"
assert any("us-" in r for r in _regions4), f"us region in {_regions4[:5]!r}..."

# Rule 5: Session has credentials-related attrs
_sess5 = boto3.Session()
assert hasattr(_sess5, "get_credentials"), "get_credentials"
assert hasattr(_sess5, "get_partition_for_region"), "get_partition_for_region"

# Rule 6: Module attributes are identity-stable
_c_ref = boto3.client
_r_ref = boto3.resource
_s_ref = boto3.Session
_sd_ref = boto3.setup_default_session
for _ in range(5):
    assert boto3.client is _c_ref, "client stable"
    assert boto3.resource is _r_ref, "resource stable"
    assert boto3.Session is _s_ref, "Session stable"
    assert boto3.setup_default_session is _sd_ref, "setup_default_session stable"

print("behavior OK")
