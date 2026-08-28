# Terraform acceptance infrastructure

The `lumen-standalone-gke` module creates one disposable, zonal Standard GKE
cluster for the Lumen standalone acceptance gate. It is test infrastructure,
not a production deployment module.

Keep Terraform state and credentials outside the repository. The acceptance
runner uses a private directory under `/private/tmp` for state, plan files,
and a task-local kubeconfig with one context. Never commit those files.

The module does not create Kubernetes objects. The Lumen CLI renders the
product StatefulSet, PVC, Service, RBAC, and NetworkPolicy. A later kustomize
harness supplies only the acceptance client jobs.
