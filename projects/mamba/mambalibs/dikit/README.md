# `mambalibs.di`

`dikit` is the internal Rust implementation area for `mambalibs.di`: reusable
provider registration, dependency resolution, request scopes, and test
overrides shared by Mamba-native libraries.

HTTP-specific adapters such as `mambalibs.http.Depends` may record a
`ProviderKey`, but generic DI behavior stays in this crate.

## Build

```bash
cargo build -p mambalibs-di
cargo build -p mambalibs-di-binding
```
