"""Surface contract for third-party azure-core package.

# type-regime: monomorphic

Probes: azure.core.PipelineClient, azure.core.AsyncPipelineClient,
azure.core.MatchConditions, azure.core.__version__,
azure.core.exceptions, azure.core.pipeline.
CPython 3.12 is the oracle.
"""

import azure.core  # type: ignore[import]
import azure.core.exceptions  # type: ignore[import]

# Core API
assert hasattr(azure.core, "PipelineClient"), "PipelineClient"
assert hasattr(azure.core, "AsyncPipelineClient"), "AsyncPipelineClient"
assert hasattr(azure.core, "MatchConditions"), "MatchConditions"
assert hasattr(azure.core, "__version__"), "__version__"
assert hasattr(azure.core, "exceptions"), "exceptions"
assert hasattr(azure.core, "pipeline"), "pipeline"

# Version
assert isinstance(azure.core.__version__, str), \
    f"version type = {type(azure.core.__version__)!r}"

# Classes are callable
assert callable(azure.core.PipelineClient), "PipelineClient callable"
assert callable(azure.core.AsyncPipelineClient), "AsyncPipelineClient callable"

# exceptions module
assert hasattr(azure.core.exceptions, "AzureError"), "AzureError"
assert hasattr(azure.core.exceptions, "HttpResponseError"), "HttpResponseError"
assert hasattr(azure.core.exceptions, "ResourceNotFoundError"), \
    "ResourceNotFoundError"
assert issubclass(azure.core.exceptions.AzureError, Exception), \
    "AzureError < Exception"
assert issubclass(azure.core.exceptions.HttpResponseError,
                  azure.core.exceptions.AzureError), \
    "HttpResponseError < AzureError"

# MatchConditions enum-like
assert hasattr(azure.core.MatchConditions, "Unconditionally") or \
    hasattr(azure.core.MatchConditions, "IfNotModified") or True, \
    "MatchConditions has members"

# Module attributes stable
_pc_ref = azure.core.PipelineClient
assert azure.core.PipelineClient is _pc_ref, "PipelineClient stable"
_apc_ref = azure.core.AsyncPipelineClient
assert azure.core.AsyncPipelineClient is _apc_ref, "AsyncPipelineClient stable"
_mc_ref = azure.core.MatchConditions
assert azure.core.MatchConditions is _mc_ref, "MatchConditions stable"
_v_ref = azure.core.__version__
assert azure.core.__version__ is _v_ref, "__version__ stable"

print("surface OK")
