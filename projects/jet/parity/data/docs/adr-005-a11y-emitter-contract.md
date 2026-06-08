# ADR-005: Semantics-to-ARIA emitter contract

| Field        | Value                                                                                  |
|--------------|----------------------------------------------------------------------------------------|
| Issue        | #2158                                                                                  |
| Parent epic  | #2136                                                                                  |
| Status       | accepted                                                                               |
| Date         | 2026-05-16                                                                             |
| Decision     | `SemanticsEmitter` trait + per-widget mapping table (`widget-role-mapping.toml`)       |

## Context

The parity track's accessibility (a11y) workstream (parent epic #2136)
splits MUI-equivalence into five sibling sub-issues, each of which
needs a *single* canonical view of "what ARIA role/state should a jet
widget produce". Today every sub-issue is forced to re-derive that
mapping ad-hoc from MUI's per-component a11y notes and the W3C ARIA
Authoring Practices Guide (APG), which (a) duplicates work,
(b) virtually guarantees per-issue drift, and (c) blocks parallel
work because every PR ends up touching the same overlapping ad-hoc
"is this the right role?" decisions.

### Why now

The parity track's first three workstreams (focus management, keyboard
navigation, color-contrast tokens) intentionally avoided AX-tree
shape, because the shape was undefined. With those three landed, every
remaining a11y sub-issue is gated on the shape question. ADR-005
unblocks all four downstream issues at once, in exchange for a single
authoring cost paid here.

### Authoring constraints

The contract must satisfy four constraints simultaneously:

* **Machine-readable.** axe-core (#2159) and the AX-tree diff (#2160)
  are automated consumers. They cannot parse free-form prose.
* **Human-reviewable.** Adding a new widget should be a one-row PR
  that an a11y reviewer can sign off on without running tooling.
* **Drift-resistant.** When MUI changes a component's a11y notes,
  there must be one and only one place in the jet repo that needs
  updating.
* **Test-anchored.** Every row must be reachable from a contract test
  so an unimplemented row fails CI rather than silently passing.

The chosen shape — a Rust trait + a TOML table + a contract test —
hits all four. See `### Why a trait, not a derive macro` for the
considered alternatives.

This ADR establishes the contract that the rest of the a11y track
consumes:

* **#2159 — axe-core CI gate.** The axe rule pack runs against jet's
  rendered AX-tree; failures are filed against widgets whose emitted
  role/state combination violates the rule set. The mapping table is
  the source-of-truth for the *expected* role/state, so axe assertions
  are not freelance — they all bottom out in this contract.
* **#2160 — AX-tree diff.** The differ compares jet's emitted AX-tree
  against the MUI baseline AX-tree on a per-region basis. The mapping
  table tells the differ which structural shape to expect; the diff
  algorithm itself (sequence alignment, fragment matching) is owned by
  #2160 and is *not* duplicated here.
* **#2161 — Live regions.** Snackbar / Alert / Badge / DataGrid update
  announcements are governed by `aria-live`, `aria-atomic`, and the
  `role=status|alert|log` triple. The required-states column in this
  table enumerates which widgets opt in.
* **#2163 — accname.** The Accessible Name Computation (accname) WPT
  suite verifies that jet's emitter produces the same accessible name
  string MUI produces. This ADR specifies *which inputs* feed accname
  (`aria-label`, `aria-labelledby`, `alt`, text content), and #2163
  owns the algorithm.

Without this contract, each of the four downstream issues drifts in a
slightly different direction and the parity track stops being a
"single source-of-truth" exercise. With it, the four issues consume a
single TOML table and a single Rust trait and can be shipped
independently.

## SemanticsEmitter trait

Every jet widget that participates in the AX-tree implements a single
trait, defined in `projects/jet/data/parity/crates/jet-a11y/src/emitter.rs`:

```rust
/// Contract every jet widget honours when producing its ARIA proxy.
///
/// One `SemanticsEmitter` instance ↔ one logical widget instance.
/// The emitter is *pure*: given the widget's current props + state, it
/// returns the AX-tree fragment to expose. The emitter does **not**
/// own DOM nodes; the framework's reconciler binds the fragment to
/// actual HTML/ARIA proxy elements.
pub trait SemanticsEmitter {
    /// The widget's primary ARIA role. Required. May be `AriaRole::None`
    /// for presentation-only sub-nodes (see composite fragments).
    fn role(&self) -> AriaRole;

    /// Per-instance ARIA state (values that change over the lifetime
    /// of the widget: `aria-checked`, `aria-expanded`, `aria-pressed`,
    /// `aria-selected`, `aria-valuenow`, ...).
    fn state(&self) -> StateMap;

    /// Per-instance ARIA properties (values fixed at construction
    /// time or via configuration: `aria-label`, `aria-labelledby`,
    /// `aria-controls`, `aria-describedby`, `aria-haspopup`,
    /// `aria-orientation`, `aria-valuemin/max`, ...).
    fn properties(&self) -> PropertyMap;

    /// The ARIA Authoring Practices Guide (APG) pattern this widget
    /// claims to implement, if any. `None` means "no APG pattern
    /// applies" (e.g. landmark wrappers like Card).
    fn apg_pattern(&self) -> Option<ApgPattern>;

    /// Composite-widget children. Empty for primitive widgets. See
    /// `## Composite widget fragment shape` for the contract on
    /// `SemanticFragment`.
    fn children(&self) -> Vec<SemanticFragment>;
}

/// Closed enum of every ARIA role jet emits. Adding a role here
/// requires updating `widget-role-mapping.toml`.
#[non_exhaustive]
pub enum AriaRole {
    None,
    Button, Checkbox, Radio, Radiogroup, Switch, Slider, Textbox,
    Combobox, Listbox, Option_, Menu, Menubar, Menuitem,
    Tab, Tablist, Tabpanel,
    Dialog, Alertdialog, Alert, Status, Tooltip, Log,
    Grid, Row, Columnheader, Rowheader, Gridcell,
    Group, Region, Navigation, List, Listitem, Toolbar, Img,
    Presentation,
}

/// `aria-*` attribute → value. Values are `AriaValue::String`,
/// `::Bool`, `::Number`, or `::IdRef` so the differ can compare
/// structurally.
pub type StateMap    = BTreeMap<AriaAttr, AriaValue>;
pub type PropertyMap = BTreeMap<AriaAttr, AriaValue>;

/// One sub-node in a composite fragment. The `purpose` field is a
/// machine-readable tag (e.g. "tab-header", "tick-mark") so the diff
/// in #2160 can match jet's children against MUI's children even if
/// the source order drifts.
pub struct SemanticFragment {
    pub role:    AriaRole,
    pub purpose: &'static str,
    pub state:      StateMap,
    pub properties: PropertyMap,
    pub children:   Vec<SemanticFragment>,
}
```

The trait is `pub` in `jet-a11y` and re-exported from the parity
top-level crate; widget crates depend on `jet-a11y` and implement
`SemanticsEmitter` directly on their public widget types.

### Why a trait, not a derive macro

A derive macro was considered and rejected. The mapping is too
data-driven (it changes whenever MUI ships a new component) to want
to round-trip through proc-macros every release. Instead the
mapping lives in TOML (this ADR's sibling deliverable), the trait
is hand-implemented, and an integration test
(`jet-a11y/tests/mapping_contract.rs`) asserts that every TOML row
has a corresponding trait impl and vice-versa.

### Lifecycle / purity contract

`SemanticsEmitter` is a *pure* trait — `&self` only, no `&mut`, no
interior mutability allowed in implementations. The emitter is called
during the framework's commit phase, possibly multiple times per
frame; any side-effects (focus shifts, live-region announcements,
DOM attribute writes) are the consumer's job, not the emitter's.

Implementations MUST be cheap: target is < 1µs per call on the
critical-render path. Allocation is permitted (the BTreeMaps are
small) but no I/O, no logging, no Rc/Arc cloning of large structures.

## Widget mapping table

The full mapping is normative in `widget-role-mapping.toml`. The
table below is the human-readable mirror; if the two disagree, the
TOML wins and the table is regenerated.

| Widget              | ARIA role     | Required states                                                                            | ARIA APG pattern          | Notes                                                              |
|---------------------|---------------|--------------------------------------------------------------------------------------------|---------------------------|--------------------------------------------------------------------|
| Button              | `button`      | `disabled?`, `aria-pressed?`, `aria-expanded?`                                              | `button`                  | `aria-expanded` only when Button opens a popup.                    |
| IconButton          | `button`      | `disabled?`, `aria-label`, `aria-pressed?`                                                  | `button`                  | `aria-label` is mandatory — no visible text.                       |
| ToggleButton        | `button`      | `aria-pressed`, `disabled?`                                                                 | `button`                  | `aria-pressed` is always present (not `?`).                        |
| ToggleButtonGroup   | `group`       | `aria-label`, `aria-orientation?`                                                           | `toolbar`                 | Composite: children are `role=button` per option.                  |
| TextField           | `textbox`     | `aria-label|aria-labelledby`, `aria-required?`, `aria-invalid?`, `aria-describedby?`, `disabled?`, `readonly?`, `aria-multiline?` | `textbox`                 | `aria-describedby` carries helper-text / error-text.               |
| Checkbox            | `checkbox`    | `aria-checked`, `disabled?`, `aria-required?`, `aria-invalid?`                              | `checkbox-mixed-checked`  | `aria-checked` is a tri-state: `true | false | "mixed"`.           |
| Radio               | `radio`       | `aria-checked`, `disabled?`                                                                 | `radio`                   | Always inside a `RadioGroup`.                                      |
| RadioGroup          | `radiogroup`  | `aria-label|aria-labelledby`, `aria-orientation?`, `aria-required?`, `aria-invalid?`        | `radio`                   | Composite: children are `role=radio` per option.                   |
| Switch              | `switch`      | `aria-checked`, `disabled?`, `aria-label|aria-labelledby`                                   | `switch`                  | NOT `checkbox` — APG distinguishes.                                |
| Slider              | `slider`      | `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext?`, `aria-orientation?`, `disabled?`, `aria-label|aria-labelledby` | `slider`                  | Composite: `role=presentation` tick-mark sub-nodes per mark.       |
| Select              | `combobox`    | `aria-expanded`, `aria-controls`, `aria-haspopup`, `aria-label|aria-labelledby`, `disabled?`, `aria-required?`, `aria-invalid?`, `aria-activedescendant?` | `combobox-select-only`    | `aria-haspopup="listbox"` — the popup is the listbox child.        |
| Menu                | `menu`        | `aria-labelledby`, `aria-orientation?`, `aria-activedescendant?`                            | `menubar`                 | Composite: `role=menuitem` per command.                            |
| MenuItem            | `menuitem`    | `disabled?`, `aria-haspopup?`, `aria-expanded?`                                             | `menubar`                 | `aria-haspopup` for submenu items.                                 |
| Tooltip             | `tooltip`     | `id`                                                                                        | `tooltip`                 | Referenced from anchor via `aria-describedby`.                     |
| Dialog              | `dialog`      | `aria-modal`, `aria-label|aria-labelledby`, `aria-describedby?`                             | `dialog-modal`            | Focus trap is enforced by #2161 live-region work.                  |
| Drawer              | `dialog`      | `aria-modal`, `aria-label|aria-labelledby`                                                  | `dialog-modal`            | Same as Dialog with side-anchored layout.                          |
| Modal               | `dialog`      | `aria-modal`, `aria-label|aria-labelledby`                                                  | `dialog-modal`            | Low-level primitive — Dialog/Drawer wrap it.                       |
| Tabs                | `tablist`     | `aria-label|aria-labelledby`, `aria-orientation?`                                           | `tabs`                    | Composite: `role=tab` per header, `role=tabpanel` per content.     |
| Tab                 | `tab`         | `aria-selected`, `aria-controls`, `disabled?`, `tabindex`                                   | `tabs`                    | `tabindex=0` on selected, `-1` on rest.                            |
| Snackbar            | `status`      | `aria-live`, `aria-atomic?`                                                                 | `alert`                   | Non-blocking announcement — owned by #2161.                        |
| Alert               | `alert`       | `aria-live?`, `aria-atomic?`                                                                | `alert`                   | Blocking announcement.                                             |
| Autocomplete        | `combobox`    | `aria-expanded`, `aria-controls`, `aria-haspopup`, `aria-autocomplete`, `aria-activedescendant?`, `aria-label|aria-labelledby`, `disabled?` | `combobox-autocomplete-list` | `aria-autocomplete="list"` or `"both"`.                            |
| DataGrid            | `grid`        | `aria-rowcount`, `aria-colcount`, `aria-multiselectable?`, `aria-readonly?`                 | `grid`                    | Minimal — virtualised rows out of scope here.                      |
| DatePicker          | `group`       | `aria-label|aria-labelledby`, `aria-haspopup`, `aria-expanded?`                             | `dialog-modal`            | Composite: textbox + button + calendar dialog.                     |
| Stepper             | `list`        | `aria-label|aria-labelledby`, `aria-orientation?`                                           | n/a                       | Composite: `role=listitem` + `role=button` per step.               |
| Pagination          | `navigation`  | `aria-label`                                                                                | n/a                       | Composite: `role=button` per page-link.                            |
| Accordion           | `region`      | `aria-labelledby`                                                                           | `accordion`               | Composite: summary `role=button` + panel `role=region`.            |
| Avatar              | `img`         | `aria-label|alt`                                                                            | n/a                       | Treated as image; falls back to initials text content.             |
| Chip                | `button`      | `disabled?`, `aria-label?`, `aria-pressed?`                                                 | `button`                  | Deletable Chip is still `role=button`; delete-affordance is a child. |
| Badge               | `status`      | `aria-label`, `aria-live?`                                                                  | n/a                       | Owned by #2161 for live-region semantics.                          |
| Card                | `region`      | `aria-labelledby?`, `aria-label?`                                                           | n/a                       | Landmark wrapper — labelling is optional.                          |

## Composite widget fragment shape

A "composite" widget is any widget whose emitted AX-tree has more
than one node. The emitter returns the root via `role()` / `state()`
/ `properties()` and the additional sub-nodes via `children()` as
`Vec<SemanticFragment>`.

Three shape rules apply:

1. **Purpose-tagged, not order-tagged.** Each `SemanticFragment`
   carries a `purpose: &'static str` discriminator (e.g.
   `"tab-header"`, `"tick-mark"`, `"page-button"`). The diff in
   #2160 matches by `purpose` before falling back to ordinal
   position so DOM-order drift doesn't false-positive.
2. **Repetition cardinality is declared in the TOML, not the trait.**
   `children` entries in `widget-role-mapping.toml` carry a `repeat`
   field with one of `"once"`, `"per-item"`, `"per-mark"`, or
   `"per-step"`. The trait's `children()` simply returns the
   instantiated list; the contract test cross-checks the cardinality
   against the TOML.
3. **Presentation children carry `role=presentation`.** Sub-nodes
   that are purely visual (Slider tick marks, decorative dividers)
   set `role = AriaRole::Presentation` so axe-core in #2159 skips
   them and the differ doesn't try to align them against semantic
   nodes.

### Worked example — Slider

```text
Slider (role=slider, aria-valuemin=0, aria-valuemax=100,
        aria-valuenow=42, aria-orientation=horizontal)
├── presentation (purpose="tick-mark", per-mark)  ×11
```

### Worked example — Tabs

```text
Tabs (role=tablist, aria-label="Settings", aria-orientation=horizontal)
├── tab     (purpose="tab-header", aria-selected=true,  aria-controls=panel-1) ×1 selected
├── tab     (purpose="tab-header", aria-selected=false, aria-controls=panel-2) ×N-1
├── tabpanel (purpose="tab-content", aria-labelledby=tab-1) ×N
```

### Worked example — Autocomplete

```text
Autocomplete (role=combobox, aria-expanded=true,
              aria-haspopup=listbox, aria-autocomplete=list,
              aria-controls=listbox-1, aria-activedescendant=option-3)
└── listbox (purpose="suggestions", once)
    └── option (purpose="suggestion", per-item, aria-selected=true on activedescendant) ×N
```

## State/property mappings

The common state/property attributes and their jet-side sources:

| ARIA attr               | Jet source                                | Notes                                                                              |
|-------------------------|-------------------------------------------|------------------------------------------------------------------------------------|
| `aria-disabled` / `disabled` | widget `disabled` prop                  | Prefer the native `disabled` HTML attribute where the proxy is an interactive element; otherwise emit `aria-disabled="true"`. |
| `aria-checked`          | widget `checked` prop (Checkbox, Radio, Switch) or `indeterminate` for `"mixed"` | Tri-state: `true | false | "mixed"`.                                               |
| `aria-selected`         | widget `selected` (Tab, Option, gridcell) | Always paired with `tabindex` management for Tab.                                  |
| `aria-expanded`         | popover/disclosure open state             | Set on the trigger, NOT the popup.                                                 |
| `aria-pressed`          | ToggleButton `pressed` prop               | `true | false | "mixed"`; absent on non-toggle Button.                             |
| `aria-controls`         | id of the element controlled              | Tab→Tabpanel, Combobox→Listbox, Disclosure→Region.                                 |
| `aria-describedby`      | id of the helper/error/tooltip element    | TextField helper-text and Tooltip both feed this.                                  |
| `aria-haspopup`         | `"menu" | "listbox" | "dialog" | "tree" | "grid"` | Required on triggers; the popup's role must match the value.                       |
| `aria-orientation`      | `"horizontal" | "vertical"`              | Optional on Tabs/Slider/Menu; default is APG-pattern-specific.                     |
| `aria-valuemin/max/now` | Slider `min` / `max` / `value`            | Numeric. `aria-valuetext` overrides screen-reader announcement for non-numeric values. |
| `aria-live`             | Snackbar / Alert / Badge severity         | `"polite"` for status, `"assertive"` for alert. Owned by #2161.                    |
| `aria-atomic`           | Live-region region semantics              | `true` when partial updates would be misleading.                                   |
| `aria-required` / `aria-invalid` | Form-control validation             | TextField, Checkbox, Radio, Switch, Select, Autocomplete.                          |
| `aria-label` / `aria-labelledby` | Accessible-name source              | At least one is required wherever the table lists `aria-label|aria-labelledby`. Computation is owned by #2163. |

### Validation expectations

Each downstream consumer applies the mapping in a different mode:

* **axe-core (#2159)** runs the AAA / WCAG 2.2 / best-practice rule
  packs against the rendered DOM. The mapping's `required_states`
  column is the must-have set; any axe failure that bottoms out on a
  state in this column blocks the parity CI gate. States flagged
  optional (`?`) are allowed to be absent in the default
  configuration but become must-have once the corresponding widget
  prop is non-default.
* **AX-tree diff (#2160)** consumes the `role` + `children` shape.
  Required-state values are compared structurally (true/false/string
  equality, with `aria-valuenow` allowed to drift inside the
  `[aria-valuemin, aria-valuemax]` window).
* **Live-regions (#2161)** consumes only the rows whose required
  states include `aria-live`. The mapping enumerates which widgets
  opt in; the politeness level (`polite` vs `assertive`) is
  determined by the widget's severity prop, not by this ADR.
* **accname (#2163)** consumes the `aria-label|aria-labelledby`
  pseudo-attribute notation in the mapping. The `|` is *not* an OR
  expressed in the AX-tree; it means "exactly one of these is the
  source of the accessible name, computed per WPT accname rules".

## Out of scope

* **Implementation of the emitter.** This ADR is contract-only. The
  actual `impl SemanticsEmitter` per widget is tracked separately
  under the parent epic #2136 and lands incrementally as widgets
  are migrated.
* **axe-core CI gate.** #2159 owns the axe-core rule pack, gating
  threshold, and CI wiring. This ADR provides only the *expected*
  role/state mapping the axe assertions consume.
* **Per-region AX-tree diff algorithm.** #2160 owns the diff
  algorithm itself (sequence alignment, fragment-purpose matching,
  noise filtering). This ADR provides only the *shape* the diff
  walks over.
* **accname WPT suite.** #2163 owns the Accessible Name
  Computation algorithm and the WPT compliance suite. This ADR
  enumerates the inputs (`aria-label`, `aria-labelledby`, `alt`,
  text content) but does not specify how they combine.
* **Live-region announcement scheduler.** #2161 owns the queue,
  debouncing, and politeness escalation for `aria-live` regions.
  This ADR only declares which widgets *opt in*.
* **High-contrast / forced-colors / reduced-motion.** These are
  visual a11y concerns; they live in a separate ADR-006 (not yet
  written) and do not flow through the semantics emitter.

## Follow-ups

1. **#2158-fu-1 — Mapping contract test.** Land an integration test
   `jet-a11y/tests/mapping_contract.rs` that loads
   `widget-role-mapping.toml` and asserts every row has a matching
   `impl SemanticsEmitter` (and vice-versa) so additions stay in
   sync.
2. **#2158-fu-2 — Codegen the `AriaRole` enum from the TOML.** Today
   the `AriaRole` enum is hand-maintained; a small `build.rs` step
   could regenerate it from the TOML so adding a new role requires
   editing one place.
3. **#2158-fu-3 — Per-locale `aria-label` defaults.** Several widgets
   (IconButton close-button, Snackbar dismiss-button) need a default
   `aria-label` that varies by locale. The mapping table currently
   says `aria-label` is required but does not specify the default
   strings; a sibling spec under `parity/schemas/aria-default-labels.toml`
   should land before #2163 closes.
4. **#2158-fu-4 — Treeview / Treegrid coverage.** MUI exposes
   `TreeView` and `TreeItem` which are not in this initial mapping.
   They map cleanly to APG `treeview` and `treegrid` patterns and
   should be added when the corresponding jet widget lands.
5. **#2158-fu-5 — Virtualised-row AX semantics for DataGrid.** The
   minimal DataGrid row mapping here doesn't cover row virtualisation
   (`aria-rowindex` on individual rows, `aria-rowcount` on the grid).
   File a separate ADR-007 for the virtualised-grid AX contract once
   #2160's diff is in place to verify it.
6. **#2158-fu-6 — Cross-check against MUI v6 release notes.** This
   ADR was authored against MUI v5 per-component a11y notes. When
   MUI v6 ships, walk every row to confirm the role/state set is
   still current; if any widget changed shape, update the TOML and
   regenerate the table.
