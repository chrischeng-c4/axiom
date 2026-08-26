---
name: lumen:build:release
description: Publish one verified Lumen release from an immutable candidate receipt.
user-invocable: true
---

# /lumen:build:release

Use this skill only after a Lumen release epic is accepted. This process has two
separate workflows. The candidate proves bytes before the tag exists. Promotion
only publishes those exact bytes after the controller creates one tag.

Never use the retired tag-first workflow. Do not push a tag to start a build.
Do not create a tag from a dirty worktree. Do not rebuild, re-sign, or re-attest
after candidate acceptance.

## Required order

1. Run release preparation. Capture `lumen@<version>` and the exact candidate
   commit.
2. Run `git:land`. Record the landed `main` SHA and merged pull request.
3. Dispatch `lumen-release-candidate` from `main`, with `version` and the exact
   landed SHA. Wait for the final v3 receipt. It must bind all nine logical job conclusions, including the isolated service and Raft library gate, the five archive pairs, two SPDX files, root and child digests, and successful native amd64 and arm64 kind gates.
4. Independently run the candidate verifier in full mode. Stop on any mismatch.
5. The controller creates one annotated `lumen@<version>` tag at the exact
   landed SHA. It must not force, move, or delete a tag. Verify the active tag
   ruleset first: target `tag`; include only `refs/tags/lumen@*`; rules exactly
   `update` and `deletion`; no bypass actor; no `creation` rule.
6. If the normal promotion has not run, dispatch `lumen-release` at that exact
   tag with `version` and `candidate_run_id`. The workflow rechecks the tag,
   ruleset, candidate run, receipt, image signature, provenance, and both SBOM
   attestations before it writes stable GHCR or GitHub Release state. If that
   tag-pinned run fails before any public write, do not rerun the known-broken
   tag route. Use `.github/workflows/lumen-release-recovery.yml` instead.
   That workflow is a fixed-identity, one-time recovery for `lumen@0.4.28`,
   and is not a generic escape hatch.
7. Run the public verifier. It must prove the annotated tag, public release
   assets and hashes, private-HOME host binary version, semver image root,
   safe `latest`, root signature, provenance, two child manifests, and both
   SPDX asset-attestation pairs.
8. Only after public verification succeeds, update and close the tracker.

## Recovery exception

The normal candidate -> tag -> promotion workflow remains canonical. A
main-only, exact-identity recovery workflow is allowed only when the
tag-pinned promotion workflow failed before any public GHCR or GitHub Release
write. The controller must reverify the same annotated tag, landed commit,
candidate receipt, digests, attestations, and ruleset before the recovery write.
The authoritative recovery workflow is
`.github/workflows/lumen-release-recovery.yml`. It is a fixed-identity,
one-time recovery for `lumen@0.4.28`, not a generic escape hatch.

Recovery cannot rebuild, move, or recreate the tag. It cannot choose a
different commit, candidate run, or digest. It cannot bypass candidate or
public verification. It must preserve the same product and candidate bytes;
the exception changes only controller tooling on `main`. The normal release
path lands the product once before candidate verification; recovery tooling
may land separately to repair the controller path, but cannot change the
frozen release commit, tag, candidate, or product bytes.

## Reruns

A rerun is allowed only for the same tag, commit, and candidate run. If the
exact public release already verifies, the promotion workflow exits success and
does not move `latest` backward. Any different receipt, digest, tag object,
release asset, candidate attempt, ruleset, or public identity is a hard stop.

## Controller boundaries

The controller owns Git, tags, GitHub workflow dispatch, tracker updates,
kind/GKE, package publication, and release closure. Workers never receive
credentials, private keys, kubeconfig, or cloud access. A missing tag ruleset
or a registry result that cannot prove a safe existing tag is a release blocker.
