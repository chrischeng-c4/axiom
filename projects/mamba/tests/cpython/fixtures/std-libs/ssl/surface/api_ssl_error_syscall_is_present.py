# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_error_syscall_is_present"
# subject = "ssl.SSL_ERROR_SYSCALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSL_ERROR_SYSCALL: api_ssl_error_syscall_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSL_ERROR_SYSCALL")
print("api_ssl_error_syscall_is_present OK")
