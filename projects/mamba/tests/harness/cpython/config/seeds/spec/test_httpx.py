# test_httpx.py — #3467 axis-1 3p httpx AssertionPass seed.
#
# Mamba-authored seed exercising the httpx surface called out in the
# issue. Uses httpx.MockTransport so the seed runs deterministically
# offline.
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like httpx, so
# `import httpx` fails on mamba today. Once mamba pkgmgr installs httpx
# cleanly and the seed flips to AssertionPass on mamba, drift detection
# prompts a `git mv spec/test_httpx.py pass/test_httpx.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + Client / Response / MockTransport surface.
#   2. Client with MockTransport — GET returns Response with status/json.
#   3. Client.post — body forwarded; method captured in mock handler.
#   4. Client headers — default headers merged into outgoing request.
#   5. Timeout — httpx.Timeout(value) round-trip.
#   6. Response.raise_for_status — no-op on 2xx, HTTPStatusError on 4xx/5xx.
#   7. URL parsing — httpx.URL splits scheme/host/port/path.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_httpx N asserts` to stdout.

import httpx

_ledger: list[int] = []

# 1. Module identity.
assert httpx.__name__ == "httpx", "httpx.__name__"
_ledger.append(1)
assert hasattr(httpx, "Client"), "httpx exposes Client"
_ledger.append(1)
assert hasattr(httpx, "Response"), "httpx exposes Response"
_ledger.append(1)
assert hasattr(httpx, "MockTransport"), "httpx exposes MockTransport"
_ledger.append(1)
assert hasattr(httpx, "URL"), "httpx exposes URL"
_ledger.append(1)


# Module-level mock handler — closes over no state.
_calls: list[tuple[str, str, dict]] = []


def _handler(request: httpx.Request) -> httpx.Response:
    _calls.append((request.method, str(request.url), dict(request.headers)))
    if request.url.path == "/ok":
        return httpx.Response(
            200,
            headers={"content-type": "application/json"},
            json={"path": "ok", "method": request.method},
        )
    if request.url.path == "/teapot":
        return httpx.Response(418, text="I'm a teapot")
    if request.url.path == "/oops":
        return httpx.Response(500, text="server error")
    return httpx.Response(404, text="missing")


_transport = httpx.MockTransport(_handler)
_client = httpx.Client(
    transport=_transport,
    base_url="http://mock",
    headers={"x-seed": "yes"},
)

# 2. GET /ok — 200 + JSON body.
_resp = _client.get("/ok")
assert isinstance(_resp, httpx.Response), "client.get returns Response"
_ledger.append(1)
assert _resp.status_code - 200 == 0, "/ok responds with 200 (boxed-dodge)"
_ledger.append(1)
assert _resp.headers["content-type"] == "application/json", (
    "Response.headers carries content-type"
)
_ledger.append(1)
_body = _resp.json()
assert _body == {"path": "ok", "method": "GET"}, "Response.json() parses payload"
_ledger.append(1)


# 3. POST /ok — method captured in mock handler.
_resp_post = _client.post("/ok", json={"x": 1})
assert _resp_post.status_code - 200 == 0, "POST /ok also 200 (boxed-dodge)"
_ledger.append(1)
assert _resp_post.json()["method"] == "POST", (
    "mock handler records method=POST"
)
_ledger.append(1)


# 4. Client default headers — x-seed header in mock-captured request.
assert len(_calls) - 2 == 0, "handler saw 2 calls so far (boxed-dodge)"
_ledger.append(1)
_first_method, _first_url, _first_headers = _calls[0]
assert _first_method == "GET", "first call was GET"
_ledger.append(1)
assert "x-seed" in _first_headers, "Client default headers propagate"
_ledger.append(1)
assert _first_headers["x-seed"] == "yes", (
    "Client default header value preserved"
)
_ledger.append(1)


# 5. httpx.Timeout — value round-trip.
_t = httpx.Timeout(5.0)
assert _t.connect == 5.0, "Timeout(5.0) sets .connect = 5.0"
_ledger.append(1)
assert _t.read == 5.0, "Timeout(5.0) sets .read = 5.0"
_ledger.append(1)
# Fine-grained Timeout — different connect / read.
_t2 = httpx.Timeout(connect=1.0, read=2.0, write=3.0, pool=4.0)
assert _t2.connect == 1.0, "fine-grained Timeout.connect"
_ledger.append(1)
assert _t2.read == 2.0, "fine-grained Timeout.read"
_ledger.append(1)


# 6. raise_for_status — 2xx no-op, 4xx + 5xx raise HTTPStatusError.
_ok = _client.get("/ok")
_raised_ok = False
try:
    _ok.raise_for_status()
except httpx.HTTPStatusError:
    _raised_ok = True
assert _raised_ok == False, "raise_for_status no-ops for 2xx"
_ledger.append(1)
_teapot = _client.get("/teapot")
_raised_418 = False
try:
    _teapot.raise_for_status()
except httpx.HTTPStatusError:
    _raised_418 = True
assert _raised_418 == True, "raise_for_status raises HTTPStatusError for 4xx"
_ledger.append(1)
assert _teapot.status_code - 418 == 0, "418 preserved (boxed-dodge)"
_ledger.append(1)
_oops = _client.get("/oops")
_raised_500 = False
try:
    _oops.raise_for_status()
except httpx.HTTPStatusError:
    _raised_500 = True
assert _raised_500 == True, "raise_for_status raises HTTPStatusError for 5xx"
_ledger.append(1)


# 7. httpx.URL parsing.
_u = httpx.URL("https://api.example.com:8443/v1/users?q=name")
assert _u.scheme == "https", "URL captures scheme"
_ledger.append(1)
assert _u.host == "api.example.com", "URL captures host"
_ledger.append(1)
assert _u.port - 8443 == 0, "URL captures port (boxed-dodge)"
_ledger.append(1)
assert _u.path == "/v1/users", "URL captures path"
_ledger.append(1)


# Cleanly close client (good citizen — also exercises Client.close()).
_client.close()

# Emit the proof-of-execution marker.
print(f"MAMBA_ASSERTION_PASS: test_httpx {len(_ledger)} asserts")
