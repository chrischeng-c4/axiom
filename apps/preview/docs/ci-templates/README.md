# Preview CI/CD templates

These templates are copyable starting points for MR-scoped UAT previews. They
are intentionally local-first: every command can be proven with `kind` before a
team connects a real GKE cluster, registry, ingress, or GitOps repository.

## Required variables

| Variable | Meaning | Becomes GKE-specific later |
|---|---|---|
| `PREVIEW_MR` | Merge request or pull request number. | No |
| `PREVIEW_SHA` | Commit SHA to deploy. | No |
| `PREVIEW_IMAGE` | Built image tag or digest. | Registry path changes. |
| `PREVIEW_APP` | Deployment, Service, and app label name. | Usually no |
| `PREVIEW_HOST` | Shared UAT host. | Yes, ingress/gateway-owned |
| `PREVIEW_BASE_NAMESPACE` | Stable UAT base namespace. | Yes |
| `PREVIEW_CONTEXT` | Kubernetes context, usually `kind-*` locally. | Yes |
| `PREVIEW_TTL_HOURS` | Cleanup TTL for preview namespaces. | Policy-owned |
| `PREVIEW_DATA_PROVIDER` | Optional data provider, currently `fake-gcp-cloud-sql` locally. | Yes |
| `PREVIEW_DATA_SOURCE_INSTANCE` | Optional base Cloud SQL/AlloyDB-style source instance. | Yes |
| `PREVIEW_DATA_DATABASE` | Optional source database name to clone/restore/seed. | Usually yes |

## Lifecycle

Open/update/rerun:

1. Build or load the image.
2. `preview discover-base`
3. `preview render`
4. Optional: `preview data plan` or `preview render --data-*`
5. Optional: `preview data apply` against fake provider state
6. `preview apply --plan-only`
7. `preview apply --dry-run`
8. `preview apply`
9. `kubectl rollout status`
10. `preview router resolve`
11. `preview comment`

Close/merge:

1. `preview cleanup plan`
2. Review protected namespace skips.
3. Optional: `preview data cleanup` for fake provider state.
4. `preview cleanup apply`
5. Re-run `preview cleanup apply` safely when the job is retried.

Use `github-actions-preview.yaml` for GitHub Actions, `gitlab-ci-preview.yml`
for GitLab CI, and `local-kind-lifecycle.sh` for an SRE laptop smoke.
