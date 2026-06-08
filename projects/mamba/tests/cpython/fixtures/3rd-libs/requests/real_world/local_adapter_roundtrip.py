"""Drive `requests.Session.get` through a transport-only mock adapter.

End-user scenario: a downstream service builds a `requests.Session`,
mounts a custom `HTTPAdapter` subclass that synthesizes a `Response`
from a hard-coded payload, and asserts status/body/headers. No
sockets, no DNS, no real HTTP — the adapter intercepts `.send()`
before requests would touch the network.

DoD: this script must exit 0 under both CPython and mamba.
"""

import io

import requests
from requests.adapters import HTTPAdapter
from requests.models import Response


class _StaticAdapter(HTTPAdapter):
    """Return a fixed Response for every request hitting `mock://`."""

    def send(self, request, stream=False, timeout=None, verify=True, cert=None, proxies=None):
        del stream, timeout, verify, cert, proxies  # mock adapter ignores transport knobs
        resp = Response()
        resp.status_code = 200
        resp.url = request.url
        resp.headers["Content-Type"] = "text/plain; charset=utf-8"
        # `Response.content` reads from `raw` lazily; using BytesIO
        # keeps the adapter independent of urllib3's pool internals.
        resp.raw = io.BytesIO(b"hello, requests")
        return resp


session = requests.Session()
session.mount("mock://", _StaticAdapter())
response = session.get("mock://example/hello")

assert response.status_code == 200, f"unexpected status: {response.status_code}"
assert response.text == "hello, requests", f"unexpected body: {response.text!r}"
assert response.headers["Content-Type"].startswith("text/plain"), (
    f"unexpected content-type: {response.headers['Content-Type']!r}"
)

print("ok:", response.status_code, response.text)
