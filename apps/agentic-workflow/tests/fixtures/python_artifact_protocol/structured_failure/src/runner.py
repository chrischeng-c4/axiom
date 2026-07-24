import json
import os
import sys

print(
    json.dumps(
        {
            "schema_version": "aw.python-artifact.result.v1",
            "status": "failed",
            "source_digest": os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"],
            "dependency_lock_digest": os.environ[
                "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"
            ],
            "evidence": ["evidence/failure.json"],
        }
    )
)
sys.exit(1)
