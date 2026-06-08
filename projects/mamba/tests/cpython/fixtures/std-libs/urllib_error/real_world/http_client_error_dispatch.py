# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "real_world"
# case = "http_client_error_dispatch"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: an HTTP client classifies failures by catching HTTPError then URLError then ContentTooShortError, extracting the right field from each (status code, reason, partial content)"""
from urllib.error import URLError, HTTPError, ContentTooShortError
import io


def classify(exc):
    """Mimic a client error handler: order matters since HTTPError and
    ContentTooShortError are both URLError subclasses."""
    if isinstance(exc, HTTPError):
        return ("http", exc.code)
    if isinstance(exc, ContentTooShortError):
        return ("short", exc.content)
    if isinstance(exc, URLError):
        return ("url", str(exc.reason))
    return ("unknown", None)


failures = [
    HTTPError("http://api/", 503, "Service Unavailable", {}, io.BytesIO(b"down")),
    ContentTooShortError("truncated", b"first-half"),
    URLError("name resolution failed"),
]

results = [classify(f) for f in failures]
assert results == [
    ("http", 503),
    ("short", b"first-half"),
    ("url", "name resolution failed"),
], results
print("http_client_error_dispatch OK")
