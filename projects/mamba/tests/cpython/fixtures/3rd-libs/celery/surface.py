"""Surface contract for third-party celery package.

# type-regime: monomorphic

Probes: celery.__version__, celery.Celery, celery.shared_task,
celery.signature, celery.result, celery.exceptions.
CPython 3.12 is the oracle.
"""

import celery  # type: ignore[import]

# Core API
assert hasattr(celery, "__version__"), "__version__"
assert hasattr(celery, "Celery"), "Celery"
assert hasattr(celery, "shared_task"), "shared_task"
assert hasattr(celery, "signature"), "signature"
assert hasattr(celery, "result"), "result"
assert hasattr(celery, "exceptions"), "exceptions"
assert hasattr(celery, "group"), "group"
assert hasattr(celery, "chain"), "chain"
assert hasattr(celery, "chord"), "chord"

# Version
assert isinstance(celery.__version__, str), \
    f"version type = {type(celery.__version__)!r}"

# Celery is callable
assert callable(celery.Celery), "Celery callable"

# Celery app construction
_app = celery.Celery("test_app")
assert hasattr(_app, "task"), "app.task"
assert hasattr(_app, "config_from_object"), "app.config_from_object"
assert hasattr(_app, "autodiscover_tasks"), "app.autodiscover_tasks"
assert hasattr(_app, "send_task"), "app.send_task"
assert _app.main == "test_app", f"app.main = {_app.main!r}"

# shared_task is callable decorator
assert callable(celery.shared_task), "shared_task callable"

# signature is callable
assert callable(celery.signature), "signature callable"

# exceptions module
import celery.exceptions as _exc  # type: ignore[import]
assert hasattr(_exc, "Retry"), "Retry"
assert hasattr(_exc, "MaxRetriesExceededError"), "MaxRetriesExceededError"
assert hasattr(_exc, "TaskRevokedError"), "TaskRevokedError"

# Module attributes stable
_v_ref = celery.__version__
assert celery.__version__ is _v_ref, "__version__ stable"
_c_ref = celery.Celery
assert celery.Celery is _c_ref, "Celery stable"
_st_ref = celery.shared_task
assert celery.shared_task is _st_ref, "shared_task stable"
_sig_ref = celery.signature
assert celery.signature is _sig_ref, "signature stable"

print("surface OK")
