# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_syscall_error_is_present"
# subject = "ssl.SSLSyscallError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLSyscallError: api_ssl_syscall_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLSyscallError")
print("api_ssl_syscall_error_is_present OK")
