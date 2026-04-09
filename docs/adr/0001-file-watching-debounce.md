# ADR 0001: File watching + debounce

**Status: Accepted**  
**Date: 09-04-2026**

## Context
`watchr` needs to watch one or more files at a time and run 
commands if a file that is being watched has been changed. 
Multiple write events in a small period of time should read as one
hence the need to debounce.

## Decision
`notify-debouncer-full` covers file watching and debounces file
events out-of-the box. 

## Alternatives Considered
- `notify`: rejected because it emits raw events that need to 
  be processed and debounced that adds unwarranted complexity

## Consequences
### Positive 
- Low boilerplate
- Reduced complexity

### Negative
- Loss of low level control
- Adding external dependency
