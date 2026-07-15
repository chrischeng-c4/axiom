---
id: scss-sass-compilation
summary: External contract for Scss Sass Compilation.
fill_sections: [e2e-test]
---

# EC: Scss Sass Compilation

---
id: scss-sass-compilation
summary: External contract for Scss Sass Compilation.
fill_sections: [e2e-test]
---

# EC: Scss Sass Compilation

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: scss-sass-compilation
    capability_id: bundler-production-build
    claim_id: scss-sass-compilation
    contract_id: scss-sass-compilation
    category: behavior
    command: "cargo test -p jet --lib css::scss"
    assertions:
      - "Jet compiles SCSS and Sass imports through the library CSS build pipeline."
```
