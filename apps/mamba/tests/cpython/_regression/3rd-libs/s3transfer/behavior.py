"""Behavior contract for third-party s3transfer package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import s3transfer  # type: ignore[import]
import s3transfer.manager  # type: ignore[import]

# Rule 1: TransferConfig defaults
_cfg1 = s3transfer.TransferConfig()
assert hasattr(_cfg1, "multipart_threshold"), "multipart_threshold"
assert hasattr(_cfg1, "multipart_chunksize"), "multipart_chunksize"
assert hasattr(_cfg1, "max_concurrency"), "max_concurrency"
assert isinstance(_cfg1.multipart_threshold, int), \
    f"threshold type = {type(_cfg1.multipart_threshold)!r}"
assert _cfg1.multipart_threshold > 0, f"threshold > 0: {_cfg1.multipart_threshold}"

# Rule 2: TransferConfig custom values are stored
_cfg2 = s3transfer.TransferConfig(
    multipart_threshold=16 * 1024 * 1024,
    max_concurrency=20,
    multipart_chunksize=16 * 1024 * 1024,
)
assert _cfg2.multipart_threshold == 16 * 1024 * 1024, \
    f"threshold = {_cfg2.multipart_threshold!r}"
assert _cfg2.max_concurrency == 20, f"concurrency = {_cfg2.max_concurrency!r}"
assert _cfg2.multipart_chunksize == 16 * 1024 * 1024, \
    f"chunksize = {_cfg2.multipart_chunksize!r}"

# Rule 3: TransferConfig max_concurrency default is reasonable
_cfg3 = s3transfer.TransferConfig()
assert 1 <= _cfg3.max_concurrency <= 100, \
    f"max_concurrency range = {_cfg3.max_concurrency!r}"

# Rule 4: TransferConfig multipart_threshold default >= 1MB
_cfg4 = s3transfer.TransferConfig()
assert _cfg4.multipart_threshold >= 1024 * 1024, \
    f"threshold >= 1MB: {_cfg4.multipart_threshold!r}"

# Rule 5: tasks module has Task base class
import s3transfer.tasks as _tasks  # type: ignore[import]
assert hasattr(_tasks, "Task"), "Task exists"
assert callable(_tasks.Task), "Task callable"

# Rule 6: Module attributes are identity-stable
_tm_ref = s3transfer.manager.TransferManager
_tc_ref = s3transfer.TransferConfig
_st_ref = s3transfer.S3Transfer
_t_ref = s3transfer.tasks
for _ in range(5):
    assert s3transfer.manager.TransferManager is _tm_ref, "TransferManager stable"
    assert s3transfer.TransferConfig is _tc_ref, "TransferConfig stable"
    assert s3transfer.S3Transfer is _st_ref, "S3Transfer stable"
    assert s3transfer.tasks is _t_ref, "tasks stable"

print("behavior OK")
