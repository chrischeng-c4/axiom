from _contract import ROOT, import_user, write_evidence


module, user = import_user(ROOT / "tech-design" / "src")
write_evidence(
    "behavior.json",
    {"class": user.__class__.__name__, "module": module.__name__, "passed": True},
)
