# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "filecookiejar_records_filename"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: a str filename is stored verbatim, a None filename is preserved as None, and an os.PathLike argument is normalized to its __fspath__ string"""
import http.cookiejar

# A str filename is recorded verbatim.
_lwp_str = http.cookiejar.LWPCookieJar("cookies.txt")
assert _lwp_str.filename == "cookies.txt", f"str filename = {_lwp_str.filename!r}"

# A None filename is preserved as None (deferred until save/load).
_lwp_none = http.cookiejar.LWPCookieJar(None)
assert _lwp_none.filename is None, f"None filename = {_lwp_none.filename!r}"


# A path-like (os.PathLike) argument is normalized to its fspath string.
class _FakePath:
    def __init__(self, p):
        self._p = p

    def __fspath__(self):
        return self._p


_lwp_path = http.cookiejar.LWPCookieJar(_FakePath("dir/cookies.txt"))
assert _lwp_path.filename == "dir/cookies.txt", f"path-like = {_lwp_path.filename!r}"

print("filecookiejar_records_filename OK")
