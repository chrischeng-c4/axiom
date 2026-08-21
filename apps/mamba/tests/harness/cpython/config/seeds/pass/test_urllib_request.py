# test_urllib_request.py — #3416 axis-1 stdlib urllib.request AssertionPass seed.
#
# Mamba-authored seed exercising the `urllib.request` module surface
# called out in the issue:
#   Request construction, headers, data, urlopen against mock —
#   no network.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. Request() with URL only — get_full_url + get_method='GET'.
#   3. Request() with data — get_method='POST' + data field exposed.
#   4. Headers — Request(headers=...) accessor + add_header/has_header.
#   5. get_header default + case-insensitive lookup.
#   6. urlopen against a `file://` URL pointing at a tempfile — true
#      "no network" mock that exercises the public urlopen API.
#
# Boxed-int dodge applied to count/length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: urllib_request N asserts` to stdout.

import urllib.request
import os
import tempfile

_ledger: list[int] = []

# 1. Module identity + public surface.
assert urllib.request.__name__ == "urllib.request", "urllib.request.__name__"
_ledger.append(1)
assert hasattr(urllib.request, "Request"), "exposes Request"
_ledger.append(1)
assert hasattr(urllib.request, "urlopen"), "exposes urlopen"
_ledger.append(1)
assert hasattr(urllib.request, "build_opener"), "exposes build_opener"
_ledger.append(1)

# 2. Request — URL only — defaults.
_req = urllib.request.Request("http://example.com/path")
assert _req.get_full_url() == "http://example.com/path", "get_full_url echoes the URL"
_ledger.append(1)
# Method defaults to GET when data is None.
assert _req.get_method() == "GET", "get_method defaults to GET (no data)"
_ledger.append(1)
# data accessor returns None when nothing was supplied.
assert _req.data is None, ".data is None when omitted"
_ledger.append(1)
# host extracted from URL.
assert _req.host == "example.com", ".host parsed from URL"
_ledger.append(1)

# 3. Request — data payload — POST + .data exposed.
_data = b'{"key":"value"}'
_req_post = urllib.request.Request("http://example.com/api", data=_data)
assert _req_post.get_method() == "POST", "get_method defaults to POST when data supplied"
_ledger.append(1)
assert _req_post.data == _data, ".data echoes the supplied payload"
_ledger.append(1)
# Explicit method override.
_req_put = urllib.request.Request(
    "http://example.com/r", data=_data, method="PUT"
)
assert _req_put.get_method() == "PUT", "explicit method='PUT' overrides POST default"
_ledger.append(1)

# 4. Request — headers constructor + add_header + has_header.
_req_h = urllib.request.Request(
    "http://example.com/", headers={"User-Agent": "mamba-test/1.0"}
)
# CPython normalises header *names* to title-case at storage.
assert _req_h.get_header("User-agent") == "mamba-test/1.0", (
    "get_header returns the constructor-supplied value (case-insensitive)"
)
_ledger.append(1)
# has_header is case-sensitive against the normalised (capitalised-first) form.
assert _req_h.has_header("User-agent") == True, "has_header on normalised name is True"
_ledger.append(1)
# Add a second header.
_req_h.add_header("Accept", "application/json")
assert _req_h.get_header("Accept") == "application/json", "add_header makes the header retrievable"
_ledger.append(1)
assert _req_h.has_header("Accept") == True, "has_header recognises the added header"
_ledger.append(1)
# Missing header returns the default.
assert _req_h.get_header("X-Missing", "default-value") == "default-value", (
    "get_header default propagates when missing"
)
_ledger.append(1)

# 5. headers attribute — dict-like with the constructor + add_header entries.
_hdrs = _req_h.headers
assert isinstance(_hdrs, dict), ".headers is a dict-like"
_ledger.append(1)
# Boxed-int dodge for the count of headers.
assert len(_hdrs) - 2 == 0, "headers dict has exactly 2 entries"
_ledger.append(1)

# 6. urlopen against a file:// URL — true "no network" mock.
_tmpdir = tempfile.mkdtemp()
_path = os.path.join(_tmpdir, "payload.txt")
_fh = open(_path, "wb")
_fh.write(b"hello from file://")
_fh.close()
_file_url = "file://" + _path
_resp = urllib.request.urlopen(_file_url)
_body = _resp.read()
_resp.close()
assert _body == b"hello from file://", "urlopen(file://) reads the file contents"
_ledger.append(1)
# urlopen returns an addinfourl-compatible object with a .url attribute.
_resp2 = urllib.request.urlopen(_file_url)
assert _resp2.url == _file_url, "urlopen response.url echoes the request URL"
_ledger.append(1)
_resp2.close()

# Cleanup the tempdir.
os.remove(_path)
os.rmdir(_tmpdir)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: urllib_request {len(_ledger)} asserts")
