# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_pathlike_object"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: guess_type accepts any os.PathLike (__fspath__) and only the final extension matters: a FakePath('LICENSE.txt') guesses identically to the string, and a directory-only path returns (None, None)"""
import mimetypes


class FakePath:
    """Minimal os.PathLike wrapper."""

    def __init__(self, value):
        self._value = value

    def __fspath__(self):
        return self._value


db = mimetypes.MimeTypes()
expected = db.guess_type("LICENSE.txt")
assert expected == ("text/plain", None), f"baseline = {expected!r}"

# A path-like object guesses identically to the equivalent string.
assert db.guess_type(FakePath("LICENSE.txt")) == expected, "plain pathlike"
assert db.guess_type(FakePath("/dir/LICENSE.txt")) == expected, "abs-dir pathlike"
assert db.guess_type(FakePath("../dir/LICENSE.txt")) == expected, "rel-dir pathlike"

# A directory-only path has no extension -> (None, None).
assert db.guess_type(FakePath("./")) == (None, None), "dir-only pathlike"
print("guess_type_pathlike_object OK")
