# ADR 0003: Add a library target (lib.rs)

**Status: Accepted**  
**Date: 25-08-2026**  

## Context
`watchr` with only a binary target (`src/main.rs` declaring 
modules) couldn't run `cargo test --doc` 
(`error: no library targets found`). So, no doc example in the 
code-base was verified by the compiler. Several examples had 
accumulated bugs (type mismatches, missing imports) that went
unnoticed as a result, and only surfaced during a documentation
cleanup pass (#39).

`watchr` is a single purpose CLI tool, not designed for external
consumption as a library - no expectation exists that another
crate would depend on it.

## Decision
Add `src/lib.rs` declaring all modules as pub. `main.rs` now 
depends on `watchr` as a library crate rather than declaring
modules directly, matching the standard Rust lib+bin split.

## Alternatives Considered
- Leave `no_run` on all examples: rejected - overclaims a 
  guarantee ("this compiles") that can't be verified without a
  library target
- Mark all examples ignore: rejected - loses compiler 
  verification of examples that are a part of the public API 
  (`read_config`, `run_init`, `run_watch`, etc).
- Do nothing: rejected - bugs in documentation would continue to
  go unnoticed

## Consequences
### Positive
- `cargo test --doc` runs and verifies public facing examples
- Internal-only functions (`pub(super)`, private) remain excluded
  from the public surface even with a lib target - no unintended
  API leakage
- Establishes a genuine public/private API boundary.

### Negative
- Examples with side effects (file I/O in `read_config`/
  `run_init`, indefinite blocking in `run_watch`) still require
  `no_run`, so verification remains partial rather than total
