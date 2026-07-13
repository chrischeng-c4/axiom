---
id: mamba-language-core-bind-instance-method-keywords
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-instance-method-keyword-applicability
entry: user instance method invoked with keyword arguments
nodes:
  lower: { kind: start, label: lower receiver method call with positional list and kwargs dict }
  inspect: { kind: process, label: resolve declared user-method parameters }
  receiver: { kind: process, label: exclude implicit self from keyword binding }
  bind: { kind: process, label: bind explicit positional and keyword values }
  dispatch: { kind: terminal, label: dispatch method and prepend receiver once }
  boundary: { kind: terminal, label: parser mangling and non-keyword calls remain unchanged }
edges:
  - { from: lower, to: inspect }
  - { from: inspect, to: receiver }
  - { from: receiver, to: bind }
  - { from: bind, to: dispatch }
  - { from: lower, to: boundary }
---
flowchart TD
    lower([lower receiver method call with positional list and kwargs dict]) --> inspect[resolve declared user-method parameters]
    inspect --> receiver[exclude implicit self from keyword binding]
    receiver --> bind[bind explicit positional and keyword values]
    bind --> dispatch([dispatch method and prepend receiver once])
    lower --> boundary([parser mangling and non-keyword calls remain unchanged])
```

`mb_call_method_kwargs` receives only explicit call arguments, while `mb_call_method` prepends the receiver during dispatch. Its binder must therefore skip the leading instance `self` parameter when resolving positional and keyword slots for a user instance method. Private-name mangling remains upstream: the AST and HIR already agree on `_Top__arg`; the binder must match that explicit keyword to the first bindable parameter without treating `self` as missing. Module functions, class dispatch, native/variadic fallback, and positional-only method calls retain their existing paths.
