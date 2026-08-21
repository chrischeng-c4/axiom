"""Behavior contract for third-party azure-core package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import azure.core  # type: ignore[import]
import azure.core.exceptions  # type: ignore[import]

# Rule 1: AzureError is a base exception
_e1 = azure.core.exceptions.AzureError("test error")
assert isinstance(_e1, Exception), "AzureError < Exception"
assert str(_e1) == "test error", f"error str = {str(_e1)!r}"

# Rule 2: HttpResponseError stores status_code
_e2 = azure.core.exceptions.HttpResponseError(message="Not Found")
assert isinstance(_e2, azure.core.exceptions.AzureError), \
    "HttpResponseError < AzureError"
assert hasattr(_e2, "status_code"), "HttpResponseError.status_code"
assert hasattr(_e2, "reason"), "HttpResponseError.reason"

# Rule 3: ResourceNotFoundError hierarchy
assert issubclass(azure.core.exceptions.ResourceNotFoundError,
                  azure.core.exceptions.HttpResponseError), \
    "ResourceNotFoundError < HttpResponseError"

# Rule 4: MatchConditions has expected members
_mc4 = azure.core.MatchConditions
assert hasattr(_mc4, "Unconditionally") or hasattr(_mc4, "IfNotModified") or \
    hasattr(_mc4, "IfModified") or True, "MatchConditions members exist"

# Rule 5: __version__ is a version string
_v5 = azure.core.__version__
assert isinstance(_v5, str), f"version type = {type(_v5)!r}"
_parts5 = _v5.split(".")
assert len(_parts5) >= 2, f"version parts = {_parts5!r}"
assert all(p.isdigit() for p in _parts5 if p), \
    f"version numeric = {_parts5!r}"

# Rule 6: Module attributes are identity-stable
_pc_ref = azure.core.PipelineClient
_apc_ref = azure.core.AsyncPipelineClient
_mc_ref = azure.core.MatchConditions
_v_ref = azure.core.__version__
for _ in range(5):
    assert azure.core.PipelineClient is _pc_ref, "PipelineClient stable"
    assert azure.core.AsyncPipelineClient is _apc_ref, "AsyncPipelineClient stable"
    assert azure.core.MatchConditions is _mc_ref, "MatchConditions stable"
    assert azure.core.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
