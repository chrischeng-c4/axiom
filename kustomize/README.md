# Kustomize acceptance harness

This directory contains only the disposable client plane for the standalone
GKE acceptance test. It does not install Lumen. The Lumen CLI remains the sole
owner of the StatefulSet, PVC, Service, RBAC, and NetworkPolicy.

The checked-in bases contain invalid sentinel values. A private run overlay
must replace them before client-side rendering or apply. Keep overlays and
credentials outside the repository.

The harness has three jobs: tooling runs first, API exercises the token
matrix, and metrics checks the internal metrics endpoint. All jobs use the
pinned curl image and a restricted, non-root security context.

Run the cloud-free contract from the repository root:

```bash
bash kustomize/lumen-standalone-acceptance/tests/contract.sh
```

The contract writes only below a task-local temporary directory. It verifies
that the checked-in bases do not change. It also rejects unsafe patches and
resource types.
