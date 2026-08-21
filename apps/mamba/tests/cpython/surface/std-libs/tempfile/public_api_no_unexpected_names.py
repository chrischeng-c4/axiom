# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "public_api_no_unexpected_names"
# subject = "tempfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile: every documented public name is present and nothing else leaks: the set of non-underscore dir(tempfile) names equals the documented public set"""
import tempfile

_public = {
    "NamedTemporaryFile", "TemporaryFile", "mkstemp", "mkdtemp", "mktemp",
    "TMP_MAX", "gettempprefix", "gettempprefixb", "gettempdir", "gettempdirb",
    "tempdir", "template", "SpooledTemporaryFile", "TemporaryDirectory",
}
for _name in _public:
    assert hasattr(tempfile, _name), f"missing public name {_name!r}"
_unexpected = [
    _k for _k in dir(tempfile) if not _k.startswith("_") and _k not in _public
]
assert _unexpected == [], f"unexpected public names = {_unexpected!r}"
print("public_api_no_unexpected_names OK")
