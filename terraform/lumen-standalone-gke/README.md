# Disposable GKE acceptance cluster

This module creates a single Standard zonal GKE cluster with one untainted
autoscaling `e2-standard-2` node pool. It enables VPC-native networking,
Dataplane V2, Workload Identity, PD CSI, and workload logs.

Use a private Terraform data directory under `/private/tmp`. Export a
task-local kubeconfig after `terraform output -raw cluster_name`; it must
contain only the context for this run. This module does not create Lumen
workloads, Kubernetes RBAC, credentials, buckets, registries, or production
networking.

Example inputs are in `terraform.tfvars.example`. `terraform apply` is
destructive test infrastructure and requires explicit operator approval.
