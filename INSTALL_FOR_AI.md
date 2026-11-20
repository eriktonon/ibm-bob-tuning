# AI Execution Runbook — Automated IBM Bob v2.1.0+ Setup

> **Audience:** AI coding assistants executing installation on a new computer on behalf of the user.
> **Rule:** Follow every step in exact order. Do **not** patch IBM Bob binaries or `dist/extension.js`. Do **not** skip verification.

---

## Step 0 — Preflight Check (Detect OS & Runtimes)

Run in the root directory of this repository (`ibm-bob-tuning`):

- **Windows (PowerShell):**
  ```powershell
  Get-Command cargo, node, git -ErrorAction SilentlyContinue | Select-Object Name, Source
  Test-Path "$env:USERPROFILE\.bob"
  ```
- **macOS / Linux (Bash):**
  ```bash
  for c in cargo node git; do command -v "$c" || echo "$c: MISSING"; done
  ls -la "$HOME/.bob" 2>/dev/null || echo "~/.bob will be created"
  ```

**Requirement:** At least **one** of `cargo` (Rust) or `node` (Node.js >= 18) must be available.
- If **both** are missing, stop and ask the user to install Node.js LTS (or Rust via rustup) before continuing.
- If `cargo` is available, the installer builds and uses the native Rust binary `bob-tuner` (recommended).
- If only `node` is available (common on locked corporate laptops), the installer automatically uses `scripts/bob-sync.mjs` + `scripts/pe-agents-ibm-mcp.mjs` with **zero `npm install`** and full parity.

---

## Step 1 — Dry-Run Preview & Repository Integrity Check

Verify the repository assets before touching `~/.bob/`:

- **Windows (PowerShell):**
  ```powershell
  node .\scripts\bob-sync.mjs validate
  .\scripts\install.ps1 -DryRun
  ```
- **macOS / Linux (Bash):**
  ```bash
  node ./scripts/bob-sync.mjs validate
  ./scripts/install.sh --dry-run
  ```

Confirm exit code `0` and `[PASS] validate: ...`.

---

## Step 2 — Apply Full Tuning (Automatic SHA-256 Backup Included)

Ask the user (or infer from current workspace) which project workspace path(s) they use in IBM Bob so `.bob/agents/*.md` is compiled into each one. If the user did not specify extra project paths, run with the current project folder (or without `-Workspace` to install all global settings/modes/skills/rules/MCPs first).

- **Windows (PowerShell):**
  ```powershell
  # Replace C:\path\to\project with the user's actual workspace path(s), or omit -Workspace for global-only
  .\scripts\install.ps1 -Workspace C:\path\to\project
  ```
- **macOS / Linux (Bash):**
  ```bash
  ./scripts/install.sh --workspace /path/to/project
  ```

### Optional Flags (only pass if requested by the user)
- `--all-modes` / `-AllModes`: puts all 23 modes into the Bob dropdown instead of the 7 curated executive modes.
- `--model <id>` / `-Model <id>`: sets `session.model` in `~/.bob/settings/settings.json` (default keeps `""` which uses Bob's standard `premium-ide` alias while pinning all 23 subagents to `model: premium`).
- `--no-turbo` / `-NoTurbo`: skips writing Turbo Mode auto-approval rules.
- `--skip-rust` / `-SkipRust`: skips Cargo compilation and forces the zero-deps Node engine immediately.

---

## Step 3 — Deterministic Post-Install Verification (Mandatory)

Verify **all 5 surfaces** after Step 2 completes:

1. **Settings & Turbo Policy (`~/.bob/settings/settings.json`):**
   - Valid JSON
   - `session.defaultMode === "pe-orquestrador"`
   - `approval.autoApprovalEnabled === true` (unless `--no-turbo`)
   - `isCommandSecurityEnabled === true`
   - `deniedCommands` contains `rm -rf`, `git push --force`, `git reset --hard`, `npm publish`, `DROP TABLE`
2. **Curated Modes (`~/.bob/settings/custom_modes.yaml`):**
   - Pure ASCII, unique slugs, valid Bob groups (`read`, `edit`, `execute`, `mcp`, `skill`, `todo`, `artifact`, `subagent`, `mode`)
3. **Global MCPs (`~/.bob/settings/mcp.json`):**
   - Valid JSON, **zero** `${...}` variable placeholders, every configured `command` path exists on disk
   - Smoke test the registered `pe-agents-ibm` MCP server over stdio (`initialize` + `tools/list` + `tools/call list_agents`)
4. **Global Rules & Skills:**
   - `~/.bob/rules/00-pe-agents-global.md` exists
   - `~/.bob/skills/*/SKILL.md` populated (47 skills)
5. **Workspace Subagents (for each target workspace):**
   - `<workspace>/.bob/agents/pe-agents-ibm-*.md` contains 23 compiled files with `model: premium` and `## Output Constraints`

Run the built-in status check:
- **Windows:** `if (Test-Path .\target\release\bob-tuner.exe) { .\target\release\bob-tuner.exe status } else { node .\scripts\bob-sync.mjs status }`
- **macOS / Linux:** `if [ -x ./target/release/bob-tuner ]; then ./target/release/bob-tuner status; else node ./scripts/bob-sync.mjs status; fi`

---

## Step 4 — Final Instructions to Report Back to the User

After Step 3 passes with exit code `0`, tell the user:
1. Show the exact SHA-256 backup path created under `~/.bob/settings/backups/`.
2. List the active MCP servers detected and registered on this computer.
3. Instruct the user to run **`Ctrl+Shift+P` (or `Cmd+Shift+P`) → `Developer: Reload Window`** inside IBM Bob IDE and open a new chat (`+`) so all MCP clients initialize.
4. Remind the user of the optional model-override test commands:
   - Test direct model: `bob-tuner set-model claude-opus-4-6` (or `node scripts/bob-sync.mjs set-model claude-opus-4-6`)
   - Revert immediately if the gateway rejects direct IDs: `bob-tuner set-model default`
