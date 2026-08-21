# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "small_block_size_warns"
# subject = "hmac.HMAC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: a caller-supplied digestmod that lacks block_size, or reports a suspiciously small one, triggers a RuntimeWarning mentioning block_size"""
import hmac
import hashlib
import warnings


class CrazyHash:
    """A digest object that (initially) has no block_size attribute."""

    def __init__(self, *args):
        self._x = hashlib.sha256(*args)
        self.digest_size = self._x.digest_size

    def update(self, v):
        self._x.update(v)

    def digest(self):
        return self._x.digest()


with warnings.catch_warnings():
    warnings.simplefilter("error", RuntimeWarning)

    # Missing block_size attribute -> RuntimeWarning.
    missing_warned = False
    try:
        hmac.HMAC(b"a", b"b", digestmod=CrazyHash)
    except RuntimeWarning as w:
        assert "block_size" in str(w), f"warn text = {str(w)!r}"
        missing_warned = True
    assert missing_warned, "missing block_size should warn"

    # A block_size that is too small -> RuntimeWarning.
    CrazyHash.block_size = 1
    small_warned = False
    try:
        hmac.HMAC(b"a", b"b", digestmod=CrazyHash)
    except RuntimeWarning as w:
        assert "block_size" in str(w), f"warn text = {str(w)!r}"
        small_warned = True
    assert small_warned, "small block_size should warn"

print("small_block_size_warns OK")
