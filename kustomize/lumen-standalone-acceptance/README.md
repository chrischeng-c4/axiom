# Lumen standalone client harness

The base is intentionally not apply-ready. It uses `INVALID_*` sentinels for
names, namespaces, run IDs, and request values. A task-local overlay replaces
only the documented metadata, identity, token, and request-value fields.

The rendered resource set is limited to one Namespace, ServiceAccounts, and
Jobs. Product resources must come from `lumen standalone gke render`.

Use `scripts/render.sh` for both static checks and live acceptance. Do not
write a second overlay generator. Every output directory must be a new path
outside the repository.

The API renderer accepts only these token and ServiceAccount pairs:

| Token mode | ServiceAccount | Default token | Projected token |
|---|---|---:|---:|
| `default` | `app` | yes | no |
| `default` | `unlisted` | yes | no |
| `projected` | `app` | no | yes |
| `projected` | `unlisted` | no | yes |
| `missing` | `default` | no | no |
| `bad` | `unlisted` | no | no |

The projected token uses the `lumen.axiom.dev` audience. The renderer fixes
its lifetime at 600 seconds. It mounts the token below
`/run/lumen/projected`.

Example:

```bash
kustomize/lumen-standalone-acceptance/scripts/render.sh api \
  --out-dir /private/tmp/lumen-api-row \
  --client-namespace lumen-client \
  --runtime-namespace lumen \
  --service lumen \
  --run-id local-check \
  --job api-default-app \
  --account app \
  --token-mode default \
  --method POST \
  --path /collections/demo/search \
  --request-file /private/tmp/request.json \
  --expected-status 2xx \
  --required-id none \
  --rejected-id none \
  --row-label default-app
```

The renderer does not contact a Kubernetes API server. The caller applies the
validated output with its own task-local kubeconfig during live acceptance.
