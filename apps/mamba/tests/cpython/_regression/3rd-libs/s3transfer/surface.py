"""Surface contract for third-party s3transfer package.

# type-regime: monomorphic

Probes: s3transfer.manager.TransferManager, s3transfer.TransferConfig,
s3transfer.S3Transfer, s3transfer.tasks, s3transfer.__version__.
CPython 3.12 is the oracle.
"""

import s3transfer  # type: ignore[import]
import s3transfer.manager  # type: ignore[import]
import s3transfer.tasks  # type: ignore[import]

# Core API
assert hasattr(s3transfer.manager, "TransferManager"), "manager.TransferManager"
assert hasattr(s3transfer, "TransferConfig"), "TransferConfig"
assert hasattr(s3transfer, "S3Transfer"), "S3Transfer"
assert hasattr(s3transfer, "tasks"), "tasks"
assert hasattr(s3transfer, "__version__"), "__version__"

# Version
assert isinstance(s3transfer.__version__, str), \
    f"version type = {type(s3transfer.__version__)!r}"

# Classes are callable
assert callable(s3transfer.manager.TransferManager), "TransferManager callable"
assert callable(s3transfer.TransferConfig), "TransferConfig callable"
assert callable(s3transfer.S3Transfer), "S3Transfer callable"

# TransferConfig construction
_cfg = s3transfer.TransferConfig(
    multipart_threshold=8 * 1024 * 1024,
    multipart_chunksize=8 * 1024 * 1024,
    max_concurrency=10,
)
assert hasattr(_cfg, "multipart_threshold"), "cfg.multipart_threshold"
assert hasattr(_cfg, "multipart_chunksize"), "cfg.multipart_chunksize"
assert hasattr(_cfg, "max_concurrency"), "cfg.max_concurrency"
assert _cfg.multipart_threshold == 8 * 1024 * 1024, \
    f"threshold = {_cfg.multipart_threshold!r}"
assert _cfg.max_concurrency == 10, f"concurrency = {_cfg.max_concurrency!r}"

# tasks module has task classes
assert hasattr(s3transfer.tasks, "Task"), "tasks.Task"

# Module attributes stable
_tm_ref = s3transfer.manager.TransferManager
assert s3transfer.manager.TransferManager is _tm_ref, "TransferManager stable"
_tc_ref = s3transfer.TransferConfig
assert s3transfer.TransferConfig is _tc_ref, "TransferConfig stable"
_st_ref = s3transfer.S3Transfer
assert s3transfer.S3Transfer is _st_ref, "S3Transfer stable"
_t_ref = s3transfer.tasks
assert s3transfer.tasks is _t_ref, "tasks stable"

print("surface OK")
