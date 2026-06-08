# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_der_cert_to_pem_cert_is_present"
# subject = "ssl.DER_cert_to_PEM_cert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.DER_cert_to_PEM_cert: api_der_cert_to_pem_cert_is_present (surface)."""
import ssl

assert hasattr(ssl, "DER_cert_to_PEM_cert")
print("api_der_cert_to_pem_cert_is_present OK")
