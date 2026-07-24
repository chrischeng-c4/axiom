import importlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "external-contracts" / "evidence"


def import_user(source_root: Path):
    sys.path.insert(0, str(source_root))
    try:
        sys.modules.pop("user_model.model", None)
        module = importlib.import_module("user_model.model")
        user = module.User()
        assert user.__class__.__name__ == "User"
        return module, user
    finally:
        sys.path.pop(0)


def write_evidence(name: str, payload: dict) -> None:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    (EVIDENCE / name).write_text(
        json.dumps(payload, sort_keys=True) + "\n",
        encoding="utf-8",
    )
