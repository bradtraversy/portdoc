# Fix: honest labels for inline scripts and macOS env fragments

**Type:** Fix (not a build-plan item)
**Status:** complete 2026-07-09. Both fixes CI-proven on ubuntu + macos; the inline-script case also verified live (python3 -c listener reports process_name python3, framework null).

## Problem

Two label-honesty defects found during 16c and 15a verification:

1. `python3 -c "import signal, ..."` labels as `import`: the runtime-script
   fallback skips flag tokens but treats the argument after an inline-code
   flag (`-c`, `-e`, `--eval`) as a script path. Labeling a process by the
   first word of its code is a guess, which the fallback rules forbid.
2. On macOS, commands can end with leaked environment fragments
   (`redis-server 127.0.0.1:6379 XPC_FLAGS=1`): processes that rewrite their
   title make sysinfo's KERN_PROCARGS2 argv parsing pick up trailing env
   entries.

## Fix

1. `src/label.rs` - `runtime_script`: scanning stops with `None` when an
   inline-code flag is hit; the row falls back to the honest interpreter
   name. Add `eval` to the identity-free script list (covers `deno eval`).
2. `src/probe/macos.rs` - trim trailing `KEY=VALUE` tokens (conservative:
   uppercase/underscore/digit keys only) from the argv before joining the
   command; never trim argv[0].

## Build steps

- [x] **Step 1 - inline-code flag guard** - fix `runtime_script` with unit
  tests: `python3 -c "import ..."` and `node -e "..."` label as None,
  `node .../bin/paperclipai run` still labels `paperclipai`. *Done when:*
  cargo test green with the new cases; a live `python3 -c` listener shows
  the interpreter name, not `import`.
- [x] **Step 2 - macOS env-tail trim** - pure `trim_env_tail` +
  `is_env_assignment` helpers in the macOS probe with unit tests (XPC_FLAGS=1
  trimmed; `--port=3000`, `127.0.0.1:6379`, argv[0] untouched). *Done when:*
  cargo test green locally (Linux side unaffected) and on the macOS CI job;
  live Redis command confirmation rides on the next Mac session.

## Testing

- Test gate on: both changes are pure logic and ship unit tests in the diff.
  Step 1 verified live on Linux; step 2's macOS tests run on the CI matrix.
