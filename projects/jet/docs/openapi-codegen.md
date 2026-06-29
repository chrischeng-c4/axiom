# `jet codegen openapi`

Generate TypeScript types, a typed client, and data-fetching hooks from an
OpenAPI 3.0/3.1 document:

```bash
jet codegen openapi ./openapi.json --out src/api
```

Output (`src/api/`): `types.ts`, `runtime.ts`, `client.ts`, `hooks.ts`,
`index.ts`. Generation is deterministic — re-running produces byte-identical
files, so the output is safe to commit and diff.

## Resolution order

Each axis (stack, HTTP runtime, hook runtime) resolves from, in priority order:

1. the CLI flag (`--stack`, `--http`, `--hooks`),
2. `[codegen.openapi]` in `jet.toml`,
3. `package.json` dependencies (auto-detection),
4. a framework-neutral default.

```toml
# jet.toml
[codegen.openapi]
stack = "react"        # auto | react | typescript
http  = "axios"        # fetch | axios
hooks = "swr"          # auto | react-query | swr | none
```

## Hook runtime: `--hooks swr | react-query | none`

`hooks.ts` exports `createHooks(client)`. The hook runtime is pluggable:

| `--hooks`     | Peer dependency             | Queries (`GET`)        | Mutations            |
|---------------|-----------------------------|------------------------|----------------------|
| `react-query` | `@tanstack/react-query`     | `useXxxQuery`          | `useXxxMutation`     |
| `swr`         | `swr` (+ `swr/mutation`)    | `useXxx` (`useSWR`)    | `useXxx` (`useSWRMutation`) |
| `none`        | —                           | not emitted            | not emitted          |

`auto` (the default) inspects `package.json` on a React stack: it picks
`react-query` when `@tanstack/react-query` is present, otherwise `swr` when
`swr` is present, otherwise emits no hooks. The hook library is a peer
dependency of the *generated output*, not of jet — only `import` statements
reference it.

### SWR

`--hooks swr` emits one `useXxx` per operation. Queries wrap `useSWR` (keyed by
`[operationId, data]`); mutations wrap `useSWRMutation` from `swr/mutation`
(keyed by `operationId`, the payload passed as `{ arg }`):

```ts
import { createClient } from "./api";
import { createHooks } from "./api/hooks";

const hooks = createHooks(createClient({ baseUrl: "/api" }));

// query
const { data, error, isLoading } = hooks.useListPets({ query: { limit: 20 } });

// mutation
const { trigger, isMutating } = hooks.useCreatePet();
await trigger({ name: "Rex" });
```

The generated `config?` argument is `SWRConfiguration` (queries) /
`SWRMutationConfiguration` (mutations), so all SWR options pass straight
through.

## HTTP runtime: `--http fetch | axios`

`runtime.ts` is the only file that changes between backends; `types.ts`,
`client.ts`, `hooks.ts`, and `index.ts` are byte-identical across `fetch` and
`axios`.

### axios version support

The axios runtime uses only `axios.create()` and
`instance.request<T>({ baseURL, url, method, params, data, headers })`,
returning `response.data`. This surface is stable across **axios `0.27.x`
(v0.x) through `1.x`** — there is no minimum major beyond `0.27`. The generated
`ClientConfig.axios` is typed `AxiosInstance`, which both lines expose.

### Injecting a pre-configured `AxiosInstance`

`ClientConfig` accepts an optional `axios` instance; when provided the client
uses it instead of `axios.create()`. This is the supported way to layer
interceptors, retries, or a cache adapter (e.g. `axios-retry`, a custom cache
adapter) onto the generated client:

```ts
import axios from "axios";
import axiosRetry from "axios-retry";
import { createClient } from "./api";

const instance = axios.create({ timeout: 10_000 });
axiosRetry(instance, { retries: 3 });
// ...attach a cache adapter, auth interceptor, etc.

const client = createClient({ baseUrl: "/api", axios: instance });
```

`baseUrl` is always applied as the per-request `baseURL`, so a shared instance
can be reused across multiple generated clients pointing at different origins.
