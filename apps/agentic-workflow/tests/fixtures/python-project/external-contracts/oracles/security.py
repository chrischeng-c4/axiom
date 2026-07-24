from _contract import ROOT, import_user, write_evidence


module, user = import_user(ROOT / "tech-design" / "src")
assert user.__class__.__module__ == "user_model.model"
write_evidence(
    "security.json",
    {"module_boundary": module.__name__, "passed": True},
)
