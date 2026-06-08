"""Behavior contract for third-party celery package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import celery  # type: ignore[import]
import celery.exceptions  # type: ignore[import]

# Rule 1: Celery app stores main name
_app1 = celery.Celery("my_worker")
assert _app1.main == "my_worker", f"main = {_app1.main!r}"

# Rule 2: Celery app config is mutable
_app2 = celery.Celery("config_app")
_app2.conf.update(task_serializer="json", result_serializer="json")
assert _app2.conf.task_serializer == "json", \
    f"task_serializer = {_app2.conf.task_serializer!r}"
assert _app2.conf.result_serializer == "json", \
    f"result_serializer = {_app2.conf.result_serializer!r}"

# Rule 3: Celery.task decorator creates a task
_app3 = celery.Celery("task_app")

@_app3.task
def _add3(x, y):
    return x + y

assert hasattr(_add3, "delay"), "task.delay"
assert hasattr(_add3, "apply_async"), "task.apply_async"
assert hasattr(_add3, "name"), "task.name"

# Rule 4: signature creates a callable signature
_sig4 = celery.signature("myapp.add", args=(1, 2), kwargs={})
assert hasattr(_sig4, "delay"), "sig.delay"
assert hasattr(_sig4, "apply_async"), "sig.apply_async"
assert hasattr(_sig4, "task"), "sig.task"
assert _sig4.task == "myapp.add", f"sig.task = {_sig4.task!r}"

# Rule 5: exceptions are accessible
assert issubclass(celery.exceptions.Retry, Exception), "Retry < Exception"
assert issubclass(celery.exceptions.MaxRetriesExceededError,
                  celery.exceptions.CeleryError), \
    "MaxRetriesExceededError < CeleryError"

# Rule 6: Module attributes are identity-stable
_v_ref = celery.__version__
_c_ref = celery.Celery
_st_ref = celery.shared_task
_sig_ref = celery.signature
for _ in range(5):
    assert celery.__version__ is _v_ref, "__version__ stable"
    assert celery.Celery is _c_ref, "Celery stable"
    assert celery.shared_task is _st_ref, "shared_task stable"
    assert celery.signature is _sig_ref, "signature stable"

print("behavior OK")
