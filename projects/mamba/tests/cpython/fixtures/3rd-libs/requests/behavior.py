"""Behavior contract for third-party requests package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import requests  # type: ignore[import]

# Rule 1: Request object holds method, url, headers correctly
_req1 = requests.Request("POST", "https://api.example.com/data",
                         headers={"Content-Type": "application/json"},
                         data=b'{"key": "value"}')
assert _req1.method == "POST", f"method = {_req1.method!r}"
assert _req1.url == "https://api.example.com/data", f"url = {_req1.url!r}"
assert _req1.headers["Content-Type"] == "application/json", "Content-Type header"
assert _req1.data == b'{"key": "value"}', f"data = {_req1.data!r}"

# Rule 2: PreparedRequest from Request has prepared attributes
_prep2 = requests.Request("GET", "http://httpbin.org/get",
                          params={"q": "test"}).prepare()
assert hasattr(_prep2, "method"), "prepared.method"
assert hasattr(_prep2, "url"), "prepared.url"
assert hasattr(_prep2, "headers"), "prepared.headers"
assert _prep2.method == "GET", f"prepared method = {_prep2.method!r}"
assert "q=test" in _prep2.url, f"params in url: {_prep2.url!r}"

# Rule 3: Session is a context manager
_results3 = []
with requests.Session() as _s3:
    _results3.append(type(_s3).__name__)
assert _results3 == ["Session"], f"session type = {_results3!r}"

# Rule 4: codes lookup works via attribute and integer
assert requests.codes.ok == 200, "codes.ok"
assert requests.codes.not_found == 404, "codes.not_found"
assert requests.codes.no_content == 204, f"codes.no_content = {requests.codes.no_content!r}"
assert requests.codes.moved_permanently == 301, \
    f"codes.moved_permanently = {requests.codes.moved_permanently!r}"

# Rule 5: HTTPError is subclass of RequestException (and IOError for compat)
assert issubclass(requests.HTTPError, requests.RequestException), \
    "HTTPError < RequestException"
assert issubclass(requests.RequestException, IOError), "RequestException < IOError"

# Rule 6: Session.headers is a dict-like object with default values
_s6 = requests.Session()
assert isinstance(_s6.headers, dict) or hasattr(_s6.headers, "__getitem__"), \
    "headers is dict-like"
# Default User-Agent header
_ua = _s6.headers.get("User-Agent", "")
assert "python-requests" in _ua.lower() or len(_ua) > 0, \
    f"User-Agent set: {_ua!r}"

# Rule 7: Session.verify is True by default (SSL verification on)
_s7 = requests.Session()
assert _s7.verify is True, f"verify default = {_s7.verify!r}"

# Rule 8: Module-level function attributes are identity-stable
_get_ref = requests.get
_post_ref = requests.post
for _ in range(5):
    assert requests.get is _get_ref, "get stable"
    assert requests.post is _post_ref, "post stable"

print("behavior OK")
