# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_pem_cert_to_der_cert_is_present"
# subject = "ssl.PEM_cert_to_DER_cert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.PEM_cert_to_DER_cert: api_pem_cert_to_der_cert_is_present (surface)."""
import ssl

assert hasattr(ssl, "PEM_cert_to_DER_cert")
print("api_pem_cert_to_der_cert_is_present OK")
