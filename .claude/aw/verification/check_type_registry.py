#!/usr/bin/env python3
"""Check the closed AW type registry and delivery facade routing."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parent))
from _paths import SCRIPTS  # noqa: E402

sys.path.insert(0, str(SCRIPTS))

import change  # noqa: E402
import wi_types  # noqa: E402
import workitem  # noqa: E402


failed: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    suffix = f" -- {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'} {label}{suffix}")
    if not ok:
        failed.append(label)


def refuses(call, phrase: str = "") -> bool:
    try:
        call()
    except (wi_types.TypeError, workitem.GhError) as exc:
        return not phrase or phrase in str(exc)
    return False


def issue(kind: str, *, state: str = "OPEN", body: str = "") -> dict:
    return {
        "number": 41,
        "title": "typed delivery",
        "body": body,
        "state": state,
        "labels": [f"type:{kind}", "app:tape", "phase:created", "priority:p2"],
        "milestone": {"number": 7, "title": "tape@0.4.1"},
        "url": "https://example.invalid/issues/41",
    }


def main() -> int:
    expected_delivery = {
        "feat": ("behavior", ("e2e", "impl")),
        "fix": ("behavior", ("e2e", "impl")),
        "refactor": ("maintenance", ("maint",)),
        "perf": ("behavior", ("e2e", "impl")),
        "test": ("maintenance", ("maint",)),
        "docs": ("maintenance", ("maint",)),
        "chore": ("maintenance", ("maint",)),
    }
    check("delivery registry is exact",
          tuple(expected_delivery) == wi_types.DELIVERY_TYPES,
          repr(wi_types.DELIVERY_TYPES))
    for kind, (flow, legs) in expected_delivery.items():
        labels = ["app:tape", f"type:{kind}"]
        check(f"type:{kind} resolves",
              wi_types.delivery_type(labels) == kind
              and wi_types.flow_for(kind) == flow
              and wi_types.required_legs(kind) == legs)

    check("intake registry is exact", wi_types.INTAKE_TYPES == ("spike", "report"))
    for kind in wi_types.INTAKE_TYPES:
        labels = [f"type:{kind}", "app:tape"]
        check(f"type:{kind} is canonical intake",
              wi_types.canonical_type(labels) == kind)
        check(f"type:{kind} cannot enter delivery",
              refuses(lambda labels=labels: wi_types.delivery_type(labels), "intake"))

    bad = {
        "zero type": ["app:tape"],
        "two types": ["type:feat", "type:fix", "app:tape"],
        "unknown type": ["type:security", "app:tape"],
    }
    for label, labels in bad.items():
        check(f"{label} is refused",
              refuses(lambda labels=labels: wi_types.canonical_type(labels)))
    for kind in wi_types.LEGACY_TYPES:
        check(f"legacy type:{kind} is refused",
              refuses(lambda kind=kind: wi_types.canonical_type([f"type:{kind}"]),
                      "retired"))

    parser = change.build_parser()
    for label, argv in (
        ("skeleton", ["skeleton"]),
        ("bodydir", ["bodydir"]),
        ("create", ["create", "--title", "x", "--body-file", "x.md"]),
    ):
        with contextlib.redirect_stderr(io.StringIO()):
            try:
                parser.parse_args(argv)
            except SystemExit as exc:
                check(f"{label} requires --type", exc.code == 2)
            else:
                check(f"{label} requires --type", False)
    body_args = parser.parse_args(["validate", "--body-file", "x.md"])
    check("body-file validation requires --type",
          refuses(lambda: change.cmd_validate(body_args), "needs --type"))
    check("normal update refuses type addition",
          refuses(lambda: workitem.reject_type_label_mutation(["type:fix"], None),
                  "cannot add"))
    check("normal update refuses type removal",
          refuses(lambda: workitem.reject_type_label_mutation(None, ["type:feat"]),
                  "cannot add"))

    original_fetch = change.fetch_issue
    original_refs = workitem.refs_commits
    original_replace = workitem.replace_issue_labels
    replacements: list[list[str]] = []
    before = issue("feat")
    after = issue("fix")
    reads = iter((before, after))
    change.fetch_issue = lambda _iid, _repo: next(reads)
    workitem.refs_commits = lambda _iid: []
    workitem.replace_issue_labels = (
        lambda _iid, _repo, labels, _dry: replacements.append(list(labels)) or ""
    )
    try:
        result = change.cmd_retype(argparse.Namespace(
            iid="41", repo="owner/repo", to="fix", dry_run=False,
        ))
        check("retype uses one complete label replacement",
              result == 0 and replacements == [[
                  "app:tape", "phase:created", "priority:p2", "type:fix"
              ]], repr(replacements))

        def retype_refused(candidate: dict, refs: list[str] | None = None) -> bool:
            replacements.clear()
            change.fetch_issue = lambda _iid, _repo: candidate
            workitem.refs_commits = lambda _iid: list(refs or [])
            return refuses(lambda: change.cmd_retype(argparse.Namespace(
                iid="41", repo="owner/repo", to="fix", dry_run=False,
            ))) and not replacements

        check("closed issue cannot retype", retype_refused(issue("feat", state="CLOSED")))
        no_created = issue("feat")
        no_created["labels"].remove("phase:created")
        check("non-created issue cannot retype", retype_refused(no_created))
        lifecycle = issue("feat", body=workitem.lifecycle_upsert(
            "", "e2e", "a" * 40, "b" * 64
        ))
        check("issue with lifecycle cannot retype", retype_refused(lifecycle))
        check("issue with delivery commit cannot retype",
              retype_refused(issue("feat"), ["c" * 40]))
        check("intake cannot use delivery retype", retype_refused(issue("report")))
    finally:
        change.fetch_issue = original_fetch
        workitem.refs_commits = original_refs
        workitem.replace_issue_labels = original_replace

    original_fetch = change.fetch_issue
    change.fetch_issue = lambda _iid, _repo: issue("perf")
    try:
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            shown = change.cmd_show(argparse.Namespace(
                iid="41", repo="owner/repo", json=True,
            ))
        payload = json.loads(stream.getvalue())
        check("show JSON reports type and flow",
              shown == 0 and payload["type"] == "perf"
              and payload["flow"] == "behavior")
    finally:
        change.fetch_issue = original_fetch

    # `is_ancestor` against the real local git checkout, unpatched: an
    # object this store never received is a routine False, an unresolvable
    # head is still a refusal, and a real head is its own ancestor.
    def is_ancestor_result(commit: str, head: str = "HEAD"):
        try:
            return workitem.is_ancestor(commit, head)
        except workitem.GhError as exc:
            return exc

    check("unknown object is not an ancestor of HEAD, no exception",
          is_ancestor_result("0" * 40) is False)
    check("a head is its own ancestor",
          is_ancestor_result("HEAD") is True)
    check("an unresolvable head still fails closed",
          refuses(lambda: workitem.is_ancestor("HEAD", "0" * 40),
                  "cannot test whether"))

    original_commit_message = workitem.commit_message
    original_commit_message_via_api = workitem.commit_message_via_api
    workitem.commit_message = lambda commit: (_ for _ in ()).throw(
        workitem.GhError(f"object {commit} not found locally"))
    workitem.commit_message_via_api = lambda commit, repo: (_ for _ in ()).throw(
        workitem.GhError(f"commits/{commit} on {repo} did not return JSON"))
    try:
        check("an unknown object never reaches a landing-proof acceptance",
              refuses(lambda: workitem.landing_proof("0" * 40, "owner/repo"),
                      "cannot read the message"))
    finally:
        workitem.commit_message = original_commit_message
        workitem.commit_message_via_api = original_commit_message_via_api

    original_fetch = change.fetch_issue
    original_refs = workitem.refs_commits
    original_message = workitem.commit_message
    original_run = workitem.run_or_show
    original_is_ancestor = workitem.is_ancestor
    original_message_via_api = workitem.commit_message_via_api
    original_default_branch = workitem.default_branch
    original_pulls_for_commit = workitem.pulls_for_commit
    original_pull_request = workitem.pull_request
    original_pull_request_commit_shas = workitem.pull_request_commit_shas
    writes: list[list[str]] = []
    proof_shas: list[str] = []

    def close_case(kind: str, evidence: list[tuple[str, str, str]],
                   *, mismatch: bool = False) -> int:
        body = ""
        messages: dict[str, str] = {}
        trailers = {
            "e2e": "E2E-Change-Digest",
            "impl": "Impl-Change-Digest",
            "maint": "Maint-Change-Digest",
        }
        for leg, sha, digest in evidence:
            body = workitem.lifecycle_upsert(body, leg, sha, digest)
            recorded = "0" * 64 if mismatch and leg == evidence[-1][0] else digest
            messages[sha] = (
                f"{leg}(wi-41): evidence\n\nRefs #41\n"
                f"{trailers[leg]}: {recorded}\n"
            )
        opened = issue(kind, body=body)
        closed = dict(opened, state="CLOSED")
        reads = iter((opened, closed))
        change.fetch_issue = lambda _iid, _repo: next(reads)
        workitem.refs_commits = lambda _iid: [sha for _leg, sha, _digest in evidence]
        workitem.commit_message = lambda sha: messages[sha]
        workitem.is_ancestor = lambda commit, head="HEAD": commit in {
            sha for _leg, sha, _digest in evidence
        }
        workitem.run_or_show = lambda argv, _dry: writes.append(list(argv)) or ""
        return change.cmd_close(argparse.Namespace(
            iid="41", repo="owner/repo", dry_run=False,
        ))

    # Route-B fixture: evidence unreachable locally and off `HEAD`'s
    # ancestry, proven landed only through a squash-merged pull request.
    # Every GitHub call the route consults is independently overridable so a
    # negative case can break exactly one condition and nothing else.
    def default_candidate(number: int = 7) -> dict:
        return {
            "number": number,
            "merged_at": "2026-09-02T18:33:00Z",
            "base": {"ref": "main", "repo": {"full_name": "owner/repo"}},
            "merge_commit_sha": "9" * 40,
        }

    def default_pr(number: int = 7) -> dict:
        return {
            "number": number,
            "merged": True,
            "base": {"ref": "main", "repo": {"full_name": "owner/repo"}},
            "merge_commit_sha": "9" * 40,
        }

    def squash_case(kind: str, evidence: list[tuple[str, str, str]], *,
                     pulls: dict[str, list[dict]] | None = None,
                     pr: dict[str, dict] | None = None,
                     pr_shas: dict[str, list[str]] | None = None,
                     ancestor_merge: str = "9" * 40,
                     api_messages: dict[str, str] | None = None,
                     local_fails: bool = True,
                     tamper_digest: bool = False,
                     tamper_refs: str | None = None,
                     pull_request_commit_shas_override=None) -> int:
        body = ""
        trailers = {
            "e2e": "E2E-Change-Digest",
            "impl": "Impl-Change-Digest",
            "maint": "Maint-Change-Digest",
        }
        messages: dict[str, str] = {}
        for leg, sha, digest in evidence:
            body = workitem.lifecycle_upsert(body, leg, sha, digest)
            recorded = ("0" * 64) if (tamper_digest and leg == evidence[-1][0]) else digest
            refs_line = tamper_refs if (tamper_refs is not None and leg == evidence[-1][0]) \
                else "Refs #41"
            messages[sha] = (
                f"{leg}(wi-41): evidence\n\n{refs_line}\n"
                f"{trailers[leg]}: {recorded}\n"
            )
        opened = issue(kind, body=body)
        closed = dict(opened, state="CLOSED")
        reads = iter((opened, closed))
        change.fetch_issue = lambda _iid, _repo: next(reads)

        known_api = dict(messages) if api_messages is None else dict(api_messages)

        def commit_message_stub(sha):
            if local_fails:
                raise workitem.GhError(f"object {sha} not found locally")
            if sha in messages:
                return messages[sha]
            raise workitem.GhError(f"object {sha} not found locally")

        def commit_message_via_api_stub(sha, _repo):
            if sha not in known_api:
                raise workitem.GhError(f"commits/{sha} resolved to a different sha")
            return known_api[sha]

        # Distinct default PR numbers per evidence sha -- reusing one number
        # for every leg would make `_pull_request_lookup`/`_shas_lookup`
        # (which resolve by number, the same shape `pull_request_commit_shas`
        # really has) unable to tell one leg's PR from another's.
        numbers = {sha: 700 + index for index, (_leg, sha, _digest) in enumerate(evidence)}
        pulls_map = pulls if pulls is not None else {
            sha: [default_candidate(numbers[sha])] for _leg, sha, _digest in evidence
        }
        pr_map = pr if pr is not None else {
            sha: default_pr(numbers[sha]) for _leg, sha, _digest in evidence
        }
        shas_map = pr_shas if pr_shas is not None else {
            sha: [sha] for _leg, sha, _digest in evidence
        }

        workitem.commit_message = commit_message_stub
        workitem.commit_message_via_api = commit_message_via_api_stub
        workitem.default_branch = lambda _repo: "main"
        workitem.pulls_for_commit = lambda sha, _repo: pulls_map.get(sha, [])
        workitem.pull_request = lambda number, _repo: _pull_request_lookup(number, pr_map)
        workitem.pull_request_commit_shas = (
            pull_request_commit_shas_override
            if pull_request_commit_shas_override is not None
            else (lambda number, _repo: _shas_lookup(number, shas_map, pr_map))
        )
        workitem.is_ancestor = lambda commit, head="HEAD": commit == ancestor_merge
        workitem.run_or_show = lambda argv, _dry: writes.append(list(argv)) or ""

        def record(commit, repo, head="HEAD"):
            proof_shas.append(commit)
            return original_landing_proof(commit, repo, head=head)

        workitem.landing_proof = record
        try:
            return change.cmd_close(argparse.Namespace(
                iid="41", repo="owner/repo", dry_run=False,
            ))
        finally:
            workitem.landing_proof = original_landing_proof

    def _pull_request_lookup(number: int, pr_map: dict[str, dict]) -> dict:
        for pr_value in pr_map.values():
            if pr_value.get("number") == number:
                return pr_value
        return {}

    def _shas_lookup(number: int, shas_map: dict[str, list[str]],
                      pr_map: dict[str, dict]) -> list[str]:
        for sha, pr_value in pr_map.items():
            if pr_value.get("number") == number:
                return shas_map.get(sha, [])
        return []

    original_landing_proof = workitem.landing_proof

    try:
        writes.clear()
        check("behavior close verifies e2e and impl evidence",
              close_case("feat", [
                  ("e2e", "a" * 40, "1" * 64),
                  ("impl", "b" * 40, "2" * 64),
              ]) == 0 and len(writes) == 1)
        writes.clear()
        check("maintenance close verifies maint evidence",
              close_case("docs", [("maint", "c" * 40, "3" * 64)]) == 0
              and len(writes) == 1)
        writes.clear()
        check("close refuses a mismatched commit digest",
              refuses(lambda: close_case(
                  "fix", [
                      ("e2e", "d" * 40, "4" * 64),
                      ("impl", "e" * 40, "5" * 64),
                  ], mismatch=True,
              ), "does not match") and not writes)
        missing_digest = issue("chore", body=workitem.lifecycle_upsert(
            "", "maint", "f" * 40, ""
        ))
        change.fetch_issue = lambda _iid, _repo: missing_digest
        check("close refuses missing lifecycle digest",
              refuses(lambda: change.cmd_close(argparse.Namespace(
                  iid="41", repo="owner/repo", dry_run=False,
              )), "missing or invalid digest"))
        wrong_flow_body = workitem.lifecycle_upsert(
            workitem.lifecycle_upsert("", "maint", "1" * 40, "6" * 64),
            "e2e", "2" * 40, "7" * 64,
        )
        change.fetch_issue = lambda _iid, _repo: issue("docs", body=wrong_flow_body)
        check("close refuses lifecycle evidence from another flow",
              refuses(lambda: change.cmd_close(argparse.Namespace(
                  iid="41", repo="owner/repo", dry_run=False,
              )), "outside this issue's required flow"))

        # Route B: squash-merged evidence, no ancestry, no local object.
        writes.clear()
        proof_shas.clear()
        check("behavior close lands through a squash-merged PR",
              squash_case("feat", [
                  ("e2e", "1" * 40, "a" * 64),
                  ("impl", "2" * 40, "b" * 64),
              ]) == 0 and len(writes) == 1
              and proof_shas == ["1" * 40, "2" * 40])

        writes.clear()
        # Mixed: e2e reachable in ancestry (route A), impl squash-merged
        # (route B).
        def mixed_ancestor(commit, head="HEAD"):
            return commit in {"3" * 40, "9" * 40}
        opened_mixed = issue("fix", body=workitem.lifecycle_upsert(
            workitem.lifecycle_upsert("", "e2e", "3" * 40, "c" * 64),
            "impl", "4" * 40, "d" * 64,
        ))
        closed_mixed = dict(opened_mixed, state="CLOSED")
        mixed_reads = iter((opened_mixed, closed_mixed))
        change.fetch_issue = lambda _iid, _repo: next(mixed_reads)
        mixed_messages = {
            "3" * 40: "e2e(wi-41): evidence\n\nRefs #41\nE2E-Change-Digest: " + "c" * 64 + "\n",
            "4" * 40: "impl(wi-41): evidence\n\nRefs #41\nImpl-Change-Digest: " + "d" * 64 + "\n",
        }

        def mixed_commit_message(sha):
            if sha == "3" * 40:
                return mixed_messages[sha]
            raise workitem.GhError(f"object {sha} not found locally")

        def mixed_commit_message_via_api(sha, _repo):
            if sha == "4" * 40:
                return mixed_messages[sha]
            raise workitem.GhError(f"commits/{sha} resolved to a different sha")

        workitem.commit_message = mixed_commit_message
        workitem.commit_message_via_api = mixed_commit_message_via_api
        workitem.is_ancestor = mixed_ancestor
        workitem.default_branch = lambda _repo: "main"
        workitem.pulls_for_commit = lambda sha, _repo: [default_candidate()] if sha == "4" * 40 else []
        workitem.pull_request = lambda number, _repo: default_pr()
        workitem.pull_request_commit_shas = lambda number, _repo: ["4" * 40]
        workitem.run_or_show = lambda argv, _dry: writes.append(list(argv)) or ""
        check("mixed ancestry + squash-merge close succeeds",
              change.cmd_close(argparse.Namespace(
                  iid="41", repo="owner/repo", dry_run=False,
              )) == 0 and len(writes) == 1)

        writes.clear()
        check("close refuses zero candidate pull requests",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pulls={"5" * 40: [], "6" * 40: [default_candidate()]}),
                  "needs exactly one") and not writes)

        writes.clear()
        two_candidates = {
            "5" * 40: [default_candidate(7), default_candidate(8)],
            "6" * 40: [default_candidate()],
        }
        check("close refuses two candidate pull requests",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pulls=two_candidates), "needs exactly one") and not writes)

        writes.clear()
        unmerged_candidate = dict(default_candidate())
        unmerged_candidate["merged_at"] = None
        check("close refuses a candidate with null merged_at",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pulls={"5" * 40: [unmerged_candidate], "6" * 40: [default_candidate()]}),
                  "needs exactly one") and not writes)

        writes.clear()
        # Default candidate numbers are 700 (e2e's sha) and 701 (impl's sha)
        # -- see `squash_case`'s `numbers` -- so a partial `pr=` override
        # must reuse those numbers or `_pull_request_lookup` resolves to the
        # wrong record.
        not_merged_pr = dict(default_pr(700))
        not_merged_pr["merged"] = False
        check("close refuses a pull request whose merged flag is false",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pr={"5" * 40: not_merged_pr, "6" * 40: default_pr(701)}),
                  "is not merged") and not writes)

        writes.clear()
        wrong_ref_pr = dict(default_pr(700))
        wrong_ref_pr["base"] = {"ref": "develop", "repo": {"full_name": "owner/repo"}}
        check("close refuses a pull request targeting the wrong branch",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pr={"5" * 40: wrong_ref_pr, "6" * 40: default_pr(701)}),
                  "does not target main") and not writes)

        writes.clear()
        wrong_repo_pr = dict(default_pr(700))
        wrong_repo_pr["base"] = {"ref": "main", "repo": {"full_name": "other/repo"}}
        check("close refuses a pull request targeting the wrong repo",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pr={"5" * 40: wrong_repo_pr, "6" * 40: default_pr(701)}),
                  "does not target owner/repo") and not writes)

        writes.clear()
        check("close refuses when the PR commits omit the evidence sha",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pr_shas={"5" * 40: ["ff" * 20], "6" * 40: ["6" * 40]}),
                  "commits do not include") and not writes)

        writes.clear()
        check("close refuses a merge commit outside HEAD's ancestry",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], ancestor_merge="8" * 40), "is not an ancestor") and not writes)

        writes.clear()

        def failing_pull_request_commit_shas(_number, _repo):
            raise workitem.GhError("gh api pulls/700/commits failed")

        check("close propagates a pull-request-commits API failure",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], pull_request_commit_shas_override=failing_pull_request_commit_shas),
                  "gh api pulls/700/commits failed") and not writes)

        writes.clear()
        check("close refuses evidence unreadable both locally and via API",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], api_messages={}), "cannot read the message") and not writes)

        writes.clear()
        check("close refuses an API sha mismatch",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], api_messages={"6" * 40:
                  "impl(wi-41): evidence\n\nRefs #41\nImpl-Change-Digest: " + "f" * 64 + "\n"}),
                  "cannot read the message") and not writes)

        writes.clear()
        check("close refuses a tampered digest read via the API",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], tamper_digest=True), "does not match") and not writes)

        writes.clear()
        check("close refuses a message missing the standalone Refs line",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], tamper_refs="see issue 41"), "standalone") and not writes)

        writes.clear()
        check("close refuses Refs appearing only inside a longer line",
              refuses(lambda: squash_case("feat", [
                  ("e2e", "5" * 40, "e" * 64),
                  ("impl", "6" * 40, "f" * 64),
              ], tamper_refs="see Refs #41 above"), "standalone") and not writes)
    finally:
        change.fetch_issue = original_fetch
        workitem.refs_commits = original_refs
        workitem.commit_message = original_message
        workitem.run_or_show = original_run
        workitem.is_ancestor = original_is_ancestor
        workitem.commit_message_via_api = original_message_via_api
        workitem.default_branch = original_default_branch
        workitem.pulls_for_commit = original_pulls_for_commit
        workitem.pull_request = original_pull_request
        workitem.pull_request_commit_shas = original_pull_request_commit_shas
        workitem.landing_proof = original_landing_proof

    if failed:
        print(f"\n=> RED: {len(failed)} failure(s)")
        return 1
    print("\n=> GREEN: type registry and delivery facade")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
