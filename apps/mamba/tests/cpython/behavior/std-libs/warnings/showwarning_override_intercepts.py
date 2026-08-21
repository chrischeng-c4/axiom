# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "showwarning_override_intercepts"
# subject = "warnings.showwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.showwarning: warn() dispatches through warnings.showwarning, so replacing the hook intercepts every warning's rendered fields; the original hook is restored afterward"""
import warnings

captured = []


def my_show(message, category, filename, lineno, file=None, line=None):
    captured.append((str(message), category.__name__))


original = warnings.showwarning
warnings.showwarning = my_show
try:
    with warnings.catch_warnings():
        warnings.simplefilter("always")
        warnings.warn("hooked", RuntimeWarning)
        warnings.warn("also hooked", FutureWarning)
finally:
    warnings.showwarning = original

assert captured == [("hooked", "RuntimeWarning"), ("also hooked", "FutureWarning")], (
    f"captured = {captured!r}"
)
# Restoring the original hook leaves the default surface intact.
assert callable(warnings.showwarning), "showwarning restored"

print("showwarning_override_intercepts OK")
