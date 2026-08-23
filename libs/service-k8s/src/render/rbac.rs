// HANDWRITE-BEGIN gap="missing-generator:logic:rbac-children" tracker="#2876,#2889" reason="Own the RBAC child shapes whose failure modes are structural — a cluster-scoped binding that must carry no owner reference and no group subject, and a namespaced Role whose every rule has to name the objects it covers — independent of any one service's policy for when they are required."
//! RBAC child objects: the cluster-scoped binding a control plane needs for
//! itself, and the namespaced Role/RoleBinding pair a service renders to hand
//! one caller a bounded grant (#2876, #2889).
//!
//! ## Cluster-scoped ([`ClusterRoleBinding`])
//!
//! Two things make it different from every other helper in
//! [`crate::render`], and both are encoded in the type rather than left to
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
//!
//! ## Namespaced ([`Role`] / [`RoleBinding`])
//!
//! These render a grant a service hands *out*, so the shape has to make the
//! narrow version the easy one.
//!
//! **Every rule names its objects.** [`NamedRule::resource_names`] is a
//! required field, not an `Option`. The difference between "create a token for
//! this one ServiceAccount" and "create a token for every ServiceAccount in
//! the namespace" is the presence of that list, and RBAC spells the second one
//! by *omission* — the dangerous grant is the one you get by not typing
//! anything. Making the field mandatory means a caller who wants the wide
//! grant has to pass an empty slice and say so.
//!
//! **A wildcard is findable.** [`first_wildcard`] walks a rendered object for
//! any string carrying a `*`, so a service can refuse to emit a grant it did
//! not mean rather than discover it in a cluster. RBAC has no other guard:
//! `verbs: ["*"]` is as valid as `verbs: ["get"]` and reads almost the same.

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

/// A subject of a namespaced [`RoleBinding`].
///
/// [`RoleSubject::User`] is an opaque authenticated-username string — whatever
/// the API server's authenticator resolved the caller to, exactly as
/// `kubectl auth whoami` prints it. Nothing here parses it or cares which
/// provider issued it: to RBAC a Google account, an OIDC subject, and a client
/// certificate's CN are the same kind of thing, and a renderer that tried to
/// tell them apart would be inventing a distinction the authorizer does not
/// make.
pub enum RoleSubject<'a> {
    User(&'a str),
    ServiceAccount(ServiceAccountSubject<'a>),
}

/// One rule of a namespaced [`Role`].
///
/// See the module doc on why `resource_names` is required rather than
/// optional.
pub struct NamedRule<'a> {
    pub api_groups: &'a [&'a str],
    pub resources: &'a [&'a str],
    /// The object names this rule is confined to.
    ///
    /// Empty renders no `resourceNames` key, which RBAC reads as *every*
    /// object of `resources` in the namespace. That is correct only when the
    /// resource is a singleton — a virtual resource standing for one
    /// namespace-wide surface — and wrong for everything else.
    pub resource_names: &'a [&'a str],
    pub verbs: &'a [&'a str],
}

impl NamedRule<'_> {
    fn render(&self) -> Value {
        let mut rule = json!({
            "apiGroups": self.api_groups,
            "resources": self.resources,
            "verbs": self.verbs,
        });
        if !self.resource_names.is_empty() {
            rule["resourceNames"] = json!(self.resource_names);
        }
        rule
    }
}

/// A namespaced set of rules.
pub struct Role<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
    pub labels: Value,
    pub rules: &'a [NamedRule<'a>],
}

/// Render `role` as a `rbac.authorization.k8s.io/v1` Role.
pub fn role(role: Role<'_>) -> Value {
    json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "Role",
        "metadata": {
            "name": role.name,
            "namespace": role.namespace,
            "labels": role.labels,
        },
        "rules": role.rules.iter().map(NamedRule::render).collect::<Vec<_>>(),
    })
}

/// A namespaced binding from a [`Role`] in the same namespace to `subjects`.
pub struct RoleBinding<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
    pub labels: Value,
    /// Name of a `Role` in `namespace`. This builder cannot bind a
    /// `ClusterRole`: doing so grants the role's rules in this namespace, and
    /// the reason to reach for it is almost always that a built-in
    /// cluster-wide role happened to contain the one verb you wanted.
    pub role: &'a str,
    pub subjects: &'a [RoleSubject<'a>],
}

/// Render `binding` as a `rbac.authorization.k8s.io/v1` RoleBinding.
pub fn role_binding(binding: RoleBinding<'_>) -> Value {
    json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "RoleBinding",
        "metadata": {
            "name": binding.name,
            "namespace": binding.namespace,
            "labels": binding.labels,
        },
        "roleRef": {
            "apiGroup": "rbac.authorization.k8s.io",
            "kind": "Role",
            "name": binding.role,
        },
        "subjects": binding.subjects.iter().map(|subject| match subject {
            RoleSubject::User(name) => json!({
                "apiGroup": "rbac.authorization.k8s.io",
                "kind": "User",
                "name": name,
            }),
            RoleSubject::ServiceAccount(sa) => json!({
                "kind": "ServiceAccount",
                "name": sa.name,
                "namespace": sa.namespace,
            }),
        }).collect::<Vec<_>>(),
    })
}

/// The path of the first string in `object` carrying a `*`, or `None`.
///
/// The path is a slash-joined trail of keys and indices (`rules/0/verbs/1`) so
/// a rejection message can name the field instead of dumping the manifest.
///
/// Deliberately a scan of the whole object rather than of the fields RBAC
/// treats as wildcards: `resources: ["pods/*"]`, `apiGroups: ["*"]`, and a
/// subject named `*` are three different mistakes, and a checker that
/// enumerated the fields it knew about would keep passing the one nobody
/// thought of.
pub fn first_wildcard(object: &Value) -> Option<String> {
    fn walk(value: &Value, path: &str) -> Option<String> {
        match value {
            Value::String(text) if text.contains('*') => Some(path.to_string()),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .find_map(|(index, item)| walk(item, &format!("{path}/{index}"))),
            Value::Object(fields) => fields
                .iter()
                .find_map(|(key, field)| walk(field, &format!("{path}/{key}"))),
            _ => None,
        }
    }
    walk(object, "").map(|path| path.trim_start_matches('/').to_string())
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

    // ---- namespaced Role / RoleBinding ----------------------------------

    /// The narrow grant is the one the type makes you write: naming the
    /// objects is a field you cannot skip, and it renders as `resourceNames`.
    #[test]
    fn a_rule_that_names_its_objects_renders_them() {
        let rules = [NamedRule {
            api_groups: &[""],
            resources: &["serviceaccounts/token"],
            resource_names: &["client"],
            verbs: &["create"],
        }];
        let obj = role(Role {
            name: "client-token-issuer",
            namespace: "team-a",
            labels: labels(),
            rules: &rules,
        });
        assert_eq!(obj["apiVersion"], "rbac.authorization.k8s.io/v1");
        assert_eq!(obj["kind"], "Role");
        assert_eq!(obj["metadata"]["namespace"], "team-a");
        assert_eq!(
            obj["rules"],
            json!([{
                "apiGroups": [""],
                "resources": ["serviceaccounts/token"],
                "verbs": ["create"],
                "resourceNames": ["client"],
            }])
        );
    }

    /// RBAC spells "every object of this resource" by leaving `resourceNames`
    /// out, so an empty slice has to render as an absent key — not as an empty
    /// list, which the apiserver reads as a rule matching nothing.
    #[test]
    fn an_empty_name_list_omits_the_key_rather_than_emitting_an_empty_one() {
        let rules = [NamedRule {
            api_groups: &["example.dev"],
            resources: &["singletons"],
            resource_names: &[],
            verbs: &["get"],
        }];
        let obj = role(Role {
            name: "r",
            namespace: "team-a",
            labels: labels(),
            rules: &rules,
        });
        let rule = obj["rules"][0].as_object().expect("a rule is an object");
        assert!(
            !rule.contains_key("resourceNames"),
            "an empty resourceNames list matches no object at all: `{rule:?}`"
        );
    }

    /// The two subject kinds render differently on purpose: a `User` needs the
    /// RBAC API group and no namespace, a ServiceAccount needs a namespace and
    /// no API group. Swapping either is accepted by the apiserver and then
    /// silently matches nobody.
    #[test]
    fn a_user_subject_and_a_service_account_subject_render_their_own_shapes() {
        let subjects = [
            RoleSubject::User("someone@example.com"),
            RoleSubject::ServiceAccount(ServiceAccountSubject {
                namespace: "team-a",
                name: "client",
            }),
        ];
        let obj = role_binding(RoleBinding {
            name: "b",
            namespace: "team-a",
            labels: labels(),
            role: "client-token-issuer",
            subjects: &subjects,
        });
        assert_eq!(obj["roleRef"]["kind"], "Role");
        assert_eq!(obj["roleRef"]["name"], "client-token-issuer");
        assert_eq!(
            obj["subjects"],
            json!([
                {
                    "apiGroup": "rbac.authorization.k8s.io",
                    "kind": "User",
                    "name": "someone@example.com",
                },
                { "kind": "ServiceAccount", "name": "client", "namespace": "team-a" },
            ])
        );
    }

    /// A username is opaque here. This one is an email, but so is a Google
    /// service account, and so are strings this renderer has never seen; the
    /// point is that none of them are parsed.
    #[test]
    fn a_user_name_is_passed_through_untouched() {
        let subjects = [RoleSubject::User(
            "lumen-client@example.iam.gserviceaccount.com",
        )];
        let obj = role_binding(RoleBinding {
            name: "b",
            namespace: "team-a",
            labels: labels(),
            role: "r",
            subjects: &subjects,
        });
        assert_eq!(
            obj["subjects"][0]["name"],
            "lumen-client@example.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn a_wildcard_is_reported_with_the_field_that_carries_it() {
        let rules = [
            NamedRule {
                api_groups: &[""],
                resources: &["serviceaccounts/token"],
                resource_names: &["client"],
                verbs: &["create"],
            },
            NamedRule {
                api_groups: &["example.dev"],
                resources: &["things"],
                resource_names: &["one"],
                verbs: &["get", "*"],
            },
        ];
        let obj = role(Role {
            name: "r",
            namespace: "team-a",
            labels: labels(),
            rules: &rules,
        });
        assert_eq!(first_wildcard(&obj).as_deref(), Some("rules/1/verbs/1"));
    }

    /// `pods/*` is a wildcard that no equality check against `"*"` would see,
    /// and it is the spelling a reviewer is least likely to notice.
    #[test]
    fn a_wildcard_inside_a_longer_string_is_still_a_wildcard() {
        let rules = [NamedRule {
            api_groups: &[""],
            resources: &["pods/*"],
            resource_names: &["one"],
            verbs: &["get"],
        }];
        let obj = role(Role {
            name: "r",
            namespace: "team-a",
            labels: labels(),
            rules: &rules,
        });
        assert_eq!(first_wildcard(&obj).as_deref(), Some("rules/0/resources/0"));
    }

    #[test]
    fn a_grant_with_no_wildcard_reports_none() {
        let rules = [NamedRule {
            api_groups: &[""],
            resources: &["serviceaccounts/token"],
            resource_names: &["client"],
            verbs: &["create"],
        }];
        let obj = role(Role {
            name: "r",
            namespace: "team-a",
            labels: labels(),
            rules: &rules,
        });
        assert_eq!(first_wildcard(&obj), None);
        let subjects = [RoleSubject::User("someone@example.com")];
        assert_eq!(
            first_wildcard(&role_binding(RoleBinding {
                name: "b",
                namespace: "team-a",
                labels: labels(),
                role: "r",
                subjects: &subjects,
            })),
            None
        );
    }
}
// HANDWRITE-END
