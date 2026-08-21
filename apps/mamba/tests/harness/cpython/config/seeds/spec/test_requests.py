# test_requests.py — #3469 axis-1 3p requests AssertionPass seed.
#
# Mamba-authored seed exercising the requests surface called out in
# the issue. Uses a custom HTTPAdapter that returns a synthetic Response
# instead of dialing the network, so the seed is fully offline.
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like requests,
# so `import requests` fails on mamba today. Once mamba pkgmgr installs
# requests cleanly and the seed flips to AssertionPass on mamba, drift
# detection prompts a
# `git mv spec/test_requests.py pass/test_requests.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + Session / Response / HTTPAdapter surface.
#   2. Session with mounted mock adapter — session.get returns synthetic
#      Response with .status_code / .headers / .text / .json().
#   3. Session.post round-trip through the same mock adapter.
#   4. Response.raise_for_status raises for 4xx/5xx, no-ops for 2xx.
#   5. Exception hierarchy: HTTPError ⊆ RequestException ⊆ IOError.
#   6. requests.utils.dict_from_cookiejar round-trips simple cookie dict.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_requests N asserts` to stdout.

import io
import json

import requests
from requests.adapters import HTTPAdapter
from requests.exceptions import HTTPError, RequestException
from requests.models import Response
from requests.utils import dict_from_cookiejar
from requests.cookies import RequestsCookieJar

_ledger: list[int] = []

# 1. Module identity.
assert requests.__name__ == "requests", "requests.__name__"
_ledger.append(1)
assert hasattr(requests, "Session"), "requests exposes Session"
_ledger.append(1)
assert hasattr(requests, "Response"), "requests exposes Response"
_ledger.append(1)
assert hasattr(requests, "__version__"), "requests has __version__"
_ledger.append(1)


# 2. Mock HTTPAdapter — returns synthetic Response keyed by URL.
class _MockAdapter(HTTPAdapter):
    """HTTPAdapter that returns canned Response objects keyed by URL."""

    def __init__(self) -> None:
        super().__init__()
        self.calls: list[tuple[str, str]] = []

    def send(self, request, **kwargs):  # type: ignore[no-untyped-def]
        self.calls.append((request.method, request.url))
        r = Response()
        # Echo URL into status/body so we can assert downstream.
        if request.url.endswith("/ok"):
            r.status_code = 200
            r.headers["Content-Type"] = "application/json"
            r._content = json.dumps({"path": "ok", "method": request.method}).encode()
            r.raw = io.BytesIO(r._content)
        elif request.url.endswith("/teapot"):
            r.status_code = 418
            r.headers["Content-Type"] = "text/plain"
            r._content = b"I'm a teapot"
            r.raw = io.BytesIO(r._content)
        elif request.url.endswith("/oops"):
            r.status_code = 500
            r.headers["Content-Type"] = "text/plain"
            r._content = b"server error"
            r.raw = io.BytesIO(r._content)
        else:
            r.status_code = 404
            r._content = b"missing"
            r.raw = io.BytesIO(r._content)
        r.url = request.url
        r.encoding = "utf-8"
        return r


_session = requests.Session()
_adapter = _MockAdapter()
_session.mount("http://mock/", _adapter)
_session.mount("https://mock/", _adapter)

# 2a. GET /ok — 200 + JSON body.
_resp = _session.get("http://mock/ok")
assert isinstance(_resp, Response), "session.get returns a Response"
_ledger.append(1)
assert _resp.status_code - 200 == 0, "/ok responds with 200 (boxed-dodge)"
_ledger.append(1)
assert _resp.headers["Content-Type"] == "application/json", (
    "Response.headers carries Content-Type"
)
_ledger.append(1)
assert _resp.text == '{"path": "ok", "method": "GET"}', (
    "Response.text decodes synthetic body via encoding"
)
_ledger.append(1)
_decoded = _resp.json()
assert _decoded == {"path": "ok", "method": "GET"}, (
    "Response.json() parses JSON body"
)
_ledger.append(1)


# 3. POST /ok — same adapter, different method recorded.
_resp_post = _session.post("http://mock/ok", json={"x": 1})
assert _resp_post.status_code - 200 == 0, "POST /ok also 200 (boxed-dodge)"
_ledger.append(1)
_dec_post = _resp_post.json()
assert _dec_post["method"] == "POST", "adapter records method for POST"
_ledger.append(1)
# Adapter saw both calls in order.
assert len(_adapter.calls) - 2 == 0, (
    "adapter saw 2 calls so far (boxed-dodge)"
)
_ledger.append(1)
assert _adapter.calls[0] == ("GET", "http://mock/ok"), "first call was GET /ok"
_ledger.append(1)
assert _adapter.calls[1] == ("POST", "http://mock/ok"), "second call was POST /ok"
_ledger.append(1)


# 4. raise_for_status — 2xx no-op, 4xx + 5xx raise.
_resp_ok = _session.get("http://mock/ok")
# 2xx → no exception.
_raised_2xx = False
try:
    _resp_ok.raise_for_status()
except HTTPError:
    _raised_2xx = True
assert _raised_2xx == False, "raise_for_status no-ops for 2xx"
_ledger.append(1)
# 4xx → HTTPError.
_resp_4xx = _session.get("http://mock/teapot")
_raised_4xx = False
try:
    _resp_4xx.raise_for_status()
except HTTPError:
    _raised_4xx = True
assert _raised_4xx == True, "raise_for_status raises HTTPError for 4xx"
_ledger.append(1)
assert _resp_4xx.status_code - 418 == 0, "418 status preserved (boxed-dodge)"
_ledger.append(1)
# 5xx → HTTPError.
_resp_5xx = _session.get("http://mock/oops")
_raised_5xx = False
try:
    _resp_5xx.raise_for_status()
except HTTPError:
    _raised_5xx = True
assert _raised_5xx == True, "raise_for_status raises HTTPError for 5xx"
_ledger.append(1)


# 5. Exception hierarchy.
assert issubclass(HTTPError, RequestException), (
    "HTTPError ⊆ RequestException"
)
_ledger.append(1)
assert issubclass(RequestException, IOError), "RequestException ⊆ IOError"
_ledger.append(1)


# 6. requests.utils.dict_from_cookiejar — round-trip simple cookies.
_jar = RequestsCookieJar()
_jar.set("session", "abc", domain="mock", path="/")
_jar.set("token", "xyz", domain="mock", path="/")
_d = dict_from_cookiejar(_jar)
assert _d.get("session") == "abc", "dict_from_cookiejar carries 'session'"
_ledger.append(1)
assert _d.get("token") == "xyz", "dict_from_cookiejar carries 'token'"
_ledger.append(1)


# Emit the proof-of-execution marker.
print(f"MAMBA_ASSERTION_PASS: test_requests {len(_ledger)} asserts")
