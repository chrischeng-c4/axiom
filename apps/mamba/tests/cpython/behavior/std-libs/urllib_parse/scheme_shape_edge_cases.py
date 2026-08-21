# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "scheme_shape_edge_cases"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: custom/unknown schemes still split netloc after '://'; opaque schemes (mailto/tel) keep their data in path; 'scheme:NN' without '//' is a path not a port; schemeless/slashless forms parse predictably"""
from urllib.parse import urlparse, urlsplit

assert urlparse("s3://foo.com/stuff") == ("s3", "foo.com", "/stuff", "", "", "")
assert urlparse("x-newscheme://foo.com/stuff?q#f") == ("x-newscheme", "foo.com", "/stuff", "", "q", "f")

assert urlparse("mailto:1337@example.org") == ("mailto", "", "1337@example.org", "", "", "")

tel = urlsplit("tel:+31-641044153")
assert tel.scheme == "tel", f"tel scheme = {tel.scheme!r}"
assert tel.path == "+31-641044153", f"tel path = {tel.path!r}"

telp = urlparse("tel:123-4;phone-context=+1-650-516")
assert telp.path == "123-4", f"tel path = {telp.path!r}"
assert telp.params == "phone-context=+1-650-516", f"tel params = {telp.params!r}"

assert urlparse("http:80") == ("http", "", "80", "", "", "")
assert urlparse("path") == ("", "", "path", "", "", "")
assert urlparse("//www.python.org:80") == ("", "www.python.org:80", "", "", "", "")

print("scheme_shape_edge_cases OK")
