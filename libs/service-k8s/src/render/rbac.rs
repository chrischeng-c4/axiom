// HANDWRITE-BEGIN gap="missing-generator:logic:cluster-scoped-rbac" tracker="#2876" reason="Own the cluster-scoped ClusterRoleBinding shape — a child that deliberately carries no owner reference and accepts only ServiceAccount subjects — independent of any one service's policy for when it is required."
//! Cluster-scoped RBAC child objects.
//!
//! Two things make these different from every other helper in
//! [`crate::render`], and both are encoded in the types rather than left to
//! each caller to remember.
//!
//! **No owner reference.** [`crate::render::RenderCtx::meta`] attaches one, and
//! for a namespaced child that is exactly right — it is what makes the child
//! disappear with its CR. A cluster-scoped object may only name a
//! cluster-scoped owner; give it a namespaced one and the garbage collector
//! does not ignore the link, it treats the owner as already gone and deletes
//! the dependent. That failure is silent, arrives minutes later, and looks
//! like an operator that cannot keep its own RBAC applied. So
//! [`ClusterRoleBinding`] takes plain labels and has no field for an owner:
//! the service's cleanup path has to be a real one.
//!
//! **Subjects are ServiceAccounts only.** The type cannot express
//! `Group/system:authenticated` or a namespace-wide ServiceAccount group,
//! which are the two subjects that turn a targeted grant into a cluster-wide
//! one. A service that genuinely needs a group subject should add it here with
//! its own reviewed constructor rather than reach it by passing a string.

use serde_json::{json, Value};

/// One ServiceAccount subject of a [`ClusterRoleBinding`], named by the
/// namespace it lives in and its own name — the pair that
/// `system:serviceaccount:<namespace>:<name>` resolves to.
pub struct ServiceAccountSubject<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
}

/// A cluster-scoped binding from `cluster_role` to `subjects`.
///
/// `cluster_role` is a name, not a definition: the intended use is binding a
/// built-in role such as `system:auth-delegator`, where authoring a
/// replacement would mean owning a copy of a grant Kubernetes already
/// maintains.
pub struct ClusterRoleBinding<'a> {
    pub name: &'a str,
    /// Recommended labels, plus whatever the service needs to resolve this
    /// object back to its owner. They are the only handle a cleanup path has —
    /// see the module doc on why there is no owner reference.
    pub labels: Value,
    pub cluster_role: &'a str,
    pub subjects: &'a [ServiceAccountSubject<'a>],
}

/// Render `binding` as a `rbac.authorization.k8s.io/v1` ClusterRoleBinding.
pub fn cluster_role_binding(binding: ClusterRoleBinding<'_>) -> Value {
    json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "ClusterRoleBinding",
        "metadata": {
            "name": binding.name,
            "labels": binding.labels,
        },
        "roleRef": {
            "apiGroup": "rbac.authorization.k8s.io",
            "kind": "ClusterRole",
            "name": binding.cluster_role,
        },
        "subjects": binding.subjects.iter().map(|s| json!({
            "kind": "ServiceAccount",
            "name": s.name,
            "namespace": s.namespace,
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Value {
        json!({ "app.kubernetes.io/managed-by": "test-operator" })
    }

    #[test]
    fn cluster_role_binding_names_the_role_and_its_service_account_subjects() {
        let subjects = [ServiceAccountSubject {
            namespace: "team-a",
            name: "svc",
        }];
        let obj = cluster_role_binding(ClusterRoleBinding {
            name: "svc-team-a-auth-delegator",
            labels: labels(),
            cluster_role: "system:auth-delegator",
            subjects: &subjects,
        });
        assert_eq!(obj["apiVersion"], "rbac.authorization.k8s.io/v1");
        assert_eq!(obj["kind"], "ClusterRoleBinding");
        assert_eq!(obj["metadata"]["name"], "svc-team-a-auth-delegator");
        assert_eq!(obj["roleRef"]["kind"], "ClusterRole");
        assert_eq!(obj["roleRef"]["name"], "system:auth-delegator");
        assert_eq!(
            obj["subjects"],
            json!([{ "kind": "ServiceAccount", "name": "svc", "namespace": "team-a" }]),
            "the subject carries its own namespace: a cluster-scoped binding has none to inherit"
        );
    }

    /// The apiserver deletes a cluster-scoped object whose owner reference
    /// names a namespaced owner, so emitting one here would be worse than
    /// emitting none. The builder has no field for it; this pins the resulting
    /// metadata so a later "just add owner refs like the other helpers" cannot
    /// pass silently.
    #[test]
    fn cluster_role_binding_carries_no_owner_reference_and_no_namespace() {
        let subjects = [ServiceAccountSubject {
            namespace: "team-a",
            name: "svc",
        }];
        let obj = cluster_role_binding(ClusterRoleBinding {
            name: "svc-team-a-auth-delegator",
            labels: labels(),
            cluster_role: "system:auth-delegator",
            subjects: &subjects,
        });
        let meta = obj["metadata"].as_object().expect("metadata is an object");
        assert!(
            !meta.contains_key("ownerReferences"),
            "a namespaced owner on a cluster-scoped object is not ignored — it is a delete order"
        );
        assert!(
            !meta.contains_key("namespace"),
            "a cluster-scoped object with a namespace is rejected by the apiserver"
        );
        assert_eq!(meta["labels"], labels());
    }

    /// Binding to a built-in role rather than to a rendered replacement is the
    /// whole point: `system:auth-delegator` is maintained by Kubernetes, and a
    /// copy of it would be a second grant to keep in sync with an upstream one
    /// nobody watches.
    #[test]
    fn cluster_role_binding_does_not_render_a_cluster_role() {
        let subjects = [ServiceAccountSubject {
            namespace: "team-a",
            name: "svc",
        }];
        let obj = cluster_role_binding(ClusterRoleBinding {
            name: "b",
            labels: labels(),
            cluster_role: "system:auth-delegator",
            subjects: &subjects,
        });
        assert!(
            obj.get("rules").is_none(),
            "this builder binds an existing role; it never defines one"
        );
    }
}
// HANDWRITE-END
