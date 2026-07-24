from _contract import ROOT, import_user, write_evidence


names = [import_user(ROOT / "src")[1].__class__.__name__ for _ in range(3)]
assert names == ["User", "User", "User"]
write_evidence("stability.json", {"imports": len(names), "passed": True})
