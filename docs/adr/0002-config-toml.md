# ADR 0002: Use TOML format for config

**Status: Accepted**  
**Date: 09-04-2026**

## Context
`watchr` uses a config file to check directories and file
extensions to watch, and commands to execute. An appropriate
config format has to be chosen that is human readable and
editable.

## Decision
TOML was chosen because of widespread existing tooling, good
support, and that it is human readable and editable.

## Alternatives Considered
- YML: rejected because of known footguns (implicity type, 
  Norway problem)

## Consequences
### Positive
- Human editable and readable
- Widespread existing tooling

### Negative
- Friction for developers coming from other ecosystems
