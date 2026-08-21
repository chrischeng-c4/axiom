# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "real_world"
# case = "status_code_response_router"
# subject = "http.client.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.responses: a response-handling router branches on http.client status constants and the responses table to dispatch a batch of (code) results into success / redirect / client-error / server-error buckets with their reason phrases, the way a client library classifies replies"""
import http.client as hc


def classify(code):
    """Bucket a status code the way a client library would, and attach
    the canonical reason phrase from http.client.responses."""
    phrase = hc.responses.get(code, "Unknown")
    if hc.OK <= code < hc.MULTIPLE_CHOICES:
        bucket = "success"
    elif hc.MULTIPLE_CHOICES <= code < hc.BAD_REQUEST:
        bucket = "redirect"
    elif hc.BAD_REQUEST <= code < hc.INTERNAL_SERVER_ERROR:
        bucket = "client_error"
    elif hc.INTERNAL_SERVER_ERROR <= code < 600:
        bucket = "server_error"
    else:
        bucket = "informational"
    return bucket, phrase


# A batch of replies a client might receive across several requests.
batch = [hc.OK, hc.CREATED, hc.MOVED_PERMANENTLY, hc.NOT_FOUND,
         hc.UNAUTHORIZED, hc.INTERNAL_SERVER_ERROR, hc.SERVICE_UNAVAILABLE]

routed = {}
for code in batch:
    bucket, phrase = classify(code)
    routed.setdefault(bucket, []).append((int(code), phrase))

assert routed["success"] == [(200, "OK"), (201, "Created")], routed["success"]
assert routed["redirect"] == [(301, "Moved Permanently")], routed["redirect"]
assert sorted(routed["client_error"]) == [(401, "Unauthorized"), (404, "Not Found")], \
    routed["client_error"]
assert sorted(routed["server_error"]) == \
    [(500, "Internal Server Error"), (503, "Service Unavailable")], routed["server_error"]
assert "informational" not in routed, "no 1xx codes in this batch"

print("status_code_response_router OK")
