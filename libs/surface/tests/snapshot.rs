// HANDWRITE-BEGIN gap="missing-generator:unit-test:surface-snapshot" tracker="pending-tracker" reason="Tests for deterministic serialized semantic surface snapshots."
use cclab_surface::{Element, Props, SurfaceRect};

#[test]
fn snapshot_serializes_stable_structural_tree() {
    let element = Element::intrinsic(
        "label",
        Props {
            id: Some("project-name-label".to_string()),
            html_for: Some("project-name".to_string()),
            ..Default::default()
        },
        vec![
            Element::text("Project"),
            Element::intrinsic(
                "input",
                Props {
                    id: Some("project-name".to_string()),
                    value: Some("aw".to_string()),
                    ..Default::default()
                },
                vec![],
            ),
        ],
    );

    let mut snapshot = element.surface_snapshot();
    assert!(snapshot.set_bounds(
        "root/1",
        SurfaceRect {
            x: 8.0,
            y: 12.0,
            w: 120.0,
            h: 24.0,
        }
    ));

    let json = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["nodes"][0]["semantic_id"], "project-name-label");
    assert_eq!(json["nodes"][0]["role"], "label");
    assert_eq!(json["nodes"][0]["name"], "Project");
    assert_eq!(json["nodes"][2]["semantic_id"], "project-name");
    assert_eq!(json["nodes"][2]["role"], "textbox");
    assert_eq!(json["nodes"][2]["name"], "aw");
    assert_eq!(json["nodes"][2]["bounds"]["w"], 120.0);
}
// HANDWRITE-END
