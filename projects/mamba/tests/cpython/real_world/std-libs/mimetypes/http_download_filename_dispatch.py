# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "real_world"
# case = "http_download_filename_dispatch"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_type: an HTTP-server-style flow classifies a batch of requested filenames via guess_type to pick a Content-Type header, falling back to application/octet-stream for unknown extensions"""
import mimetypes


def content_type_for(path):
    """Pick a Content-Type header for a download, like a static file server."""
    mime, encoding = mimetypes.guess_type(path)
    if mime is None:
        return "application/octet-stream", encoding
    return mime, encoding


requests = [
    ("/static/index.html", "text/html", None),
    ("/assets/app.css", "text/css", None),
    ("/assets/bundle.js", "text/javascript", None),
    ("/img/logo.png", "image/png", None),
    ("/data/report.json", "application/json", None),
    ("/dl/release.tar.gz", "application/x-tar", "gzip"),
    ("/dl/unknown.bespoke_blob", "application/octet-stream", None),
]
for path, expected_type, expected_enc in requests:
    ctype, enc = content_type_for(path)
    assert ctype == expected_type, f"{path!r}: type = {ctype!r}, expected {expected_type!r}"
    assert enc == expected_enc, f"{path!r}: encoding = {enc!r}, expected {expected_enc!r}"
print("http_download_filename_dispatch OK")
