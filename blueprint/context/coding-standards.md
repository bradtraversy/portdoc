# Coding Standards

> Conventions for PortDoc: a Rust (axum + clap + tokio) binary in `src/` and a
> React + TypeScript + Vite frontend in `web/`. Edit as the project reveals
> real patterns; `> TODO` marks conventions not yet decided.

## Rust (backend)

- Edition 2024, formatted with `cargo fmt` (default rustfmt settings)
- `cargo clippy` clean - fix warnings, don't allow-list them without a reason
- `unwrap`/`expect` only at startup where dying is correct (like binding the
  listener); fallible runtime paths return errors instead of panicking
- Async runtime is tokio; the server is axum with routes built on `Router`
- CLI flags go through the clap `Cli` derive struct in `main.rs`

- Errors: `thiserror` for typed error enums (decided at the probe boundary,
  feature 4); fallible paths return these, `anyhow` stays out of the codebase
- Module layout: flat modules off `main.rs` (`snapshot`, `mock`); platform
  code splits per OS inside a module folder (`probe/mod.rs` boundary +
  `probe/linux.rs`); split further only when a feature needs it

## TypeScript (frontend)

- Strict mode enabled (Vite template tsconfig)
- No `any` types - use proper typing or `unknown`
- Define interfaces for props, API responses, and shared data models
- Use type inference where obvious, explicit types where helpful

## React

- Functional components only, hooks for state and side effects
- Keep components focused - one job per component
- Extract reusable logic into custom hooks
- Lint with oxlint (`npm run lint` in `web/`); `react/rules-of-hooks` is an error

## File Organization

- Rust: `src/` at the repo root, single binary crate
- Frontend app code: `web/src/` (components, styles, assets)
- Static assets served as-is: `web/public/`
- Production frontend build: `web/dist` (generated, ignored)

- Component folder structure: Keep `web/src/components/` flat for views and domain components. Reusable/generic UI components belong in `web/src/components/ui/`.

## Naming

- Rust: modules and functions `snake_case`, types `PascalCase`, constants
  `SCREAMING_SNAKE_CASE`
- React components: PascalCase (`PortCard.tsx`), files match the component name
- TS functions and variables: camelCase; types/interfaces PascalCase, no prefix

## Styling

- UI stack (installed at build item 3): Tailwind CSS v4 via `@tailwindcss/vite`,
  hand-built shadcn/ui-style components in `web/src/components/ui/`, TanStack
  Table for the services table, Lucide icons (`lucide-react`)
- Theme tokens live in the `@theme` block of `web/src/index.css` (ported from
  the prototypes): graphite surfaces, amber accent for interaction only, status
  colors always paired with a dot or text label, sans UI type + mono for data
  (ports, PIDs, commands, paths)
- No inline styles

## API Boundary

- The axum server owns system facts (running processes, ports, project mapping);
  the React app renders them
- The frontend talks to the backend over HTTP endpoints served by the binary

- The backend serves API endpoints under the `/api/` prefix (e.g., `/api/snapshot`, `/api/health`).
- The JSON contract is strictly typed by `DevSnapshot` and related structures.
- Production builds embed `web/dist` inside the binary; debug builds read it dynamically from disk.

## Error Handling

- Backend: handlers return proper HTTP status codes, never panic on bad input
- Frontend: surface fetch failures in the UI instead of failing silently

## Testing

The blueprint installs no test runner; testing is opt-in at the project level,
because the overlay can't know your stack. Adding unit testing is an explicit
setup task the AI can do through the normal workflow, either as a build-plan item
or with `/fix "add unit testing"`. The setup should choose the stack-native
runner, wire the scripts or commands, add a small example test, and update the
Commands section of `AGENTS.md`.

**The opt-in switch is one signal: a `test` command in the Commands section of
`AGENTS.md`.** Declare one and **tests become a gate for logic-bearing steps**,
not an optional extra; leave it out and the loop verifies logic with the evidence
it already uses (run it, a screenshot, the build). Adding the runner is itself a
deliberate step, never a silent mid-step install. This is the single definition
of the switch; the skills and `ai-interaction.md` only point back here.

- **What to test (the scope rule):** pure logic where a wrong answer is possible -
  parsers, formatters, validators, port/process mapping logic. These have
  assertable inputs and outputs and real edge cases (empty, missing, malformed).
- **What not to test:** UI components and integration-level surfaces (anything
  driving a real browser, live sockets, or the process table). Verify those with
  a screenshot and the build, not brittle unit tests.
- **The gate (when a runner is configured):** a build step that adds in-scope logic
  must ship a passing test in the same reviewable diff. The project's test command
  must be green before the step is approved, before any checkpoint commit, and
  before `/complete` merges. UI and integration-only steps are exempt and ride on
  screenshot plus build evidence.
- **When it's named:** the `/feature` spec's Testing section predicts the coverage,
  `/implement` writes the test with the step, and if a step surfaces logic the spec
  didn't foresee, add a focused test then.
- An empty suite should fail, not pass, so "no tests ran" never looks like "passed".
- Test files live next to source files (Rust: `#[cfg(test)]` modules in the same
  file; TS: `feature.test.ts` beside `feature.ts`).
- Run them via the project's test command (see Commands in `AGENTS.md`), not a
  hardcoded tool name.

Stack binding: the Rust side uses `cargo test` with inline `#[cfg(test)]`
modules, and the gate is **on**: `cargo test` is declared in the Commands
section of `AGENTS.md`. The frontend has no runner and would use Vitest if it
ever grows testable logic; until then UI work rides on screenshot plus build
evidence.

## Browser Verification

For UI and integration behavior, prefer real browser evidence over reading the
code and assuming it works.

- If Playwright is already installed, or the Commands section of `AGENTS.md`
  declares a Playwright script, use Playwright for browser checks, screenshots,
  console-error checks, and user-flow verification.
- If Playwright is not installed, do not add it silently in the middle of an
  unrelated feature. Use the available dev server, browser screenshots, build
  output, API output, or manual verification evidence instead.
- Add Playwright only when the user asks for it, or when the current spec is
  explicitly about setting up browser automation.
- Browser evidence is especially important for flows that click, type, submit,
  navigate, download files, render complex layouts, or depend on client-side
  state.

## Code Quality

- No commented-out code unless specified
- No unused imports or variables
- Keep functions under 50 lines when possible

## Comments

Write code that explains itself; comment only what the code cannot say.
Over-commenting is a common AI tell, so resist it.

- Comment the **why**, not the **what**. Delete any comment that restates the code.
- No banner/header blocks, section dividers, or step-by-step narration of obvious
  code. A file does not need a comment announcing each region.
- A comment earns its place only when it captures something the code can't: a
  non-obvious decision, a gotcha or workaround, why a value is what it is, or a
  link to a spec or issue.
- Prefer self-documenting names and small functions over explanatory comments.
- Keep doc comments minimal: a one-line purpose on an exported type or function is
  plenty; don't write JSDoc that just repeats the signature.
- When in doubt, leave the comment out.

## Writing

- No em dashes (U+2014) in generated content: docs, comments, commit messages,
  READMEs, specs. They read as AI-generated.
- Use a hyphen for `term - description` separators; rephrase prose with commas,
  parentheses, or a colon. Avoid en dashes and the ellipsis character too.
