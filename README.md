# IBM Bob Tuning (`PE-AGENTS-IBM` Edition)

Standalone, cross-platform tuning kit exclusively built for **IBM Bob IDE v2.1.0+** (`bob-code`), engineered from static analysis of Bob's actual configuration loaders, `ApprovalEngine`, `WorkspaceAgentsManager`, `WorkspaceModeManager`, and `McpConfigManager`.

Zero clutter from other IDEs. Zero hardcoded user paths. Works on **Windows, macOS, and Linux** — using either the native Rust CLI (`bob-tuner`) or the built-in zero-dependency Node.js fallback (`scripts/bob-sync.mjs`) when Cargo is not installed (e.g., locked corporate laptops).

---

## What This Tuning Fixes in IBM Bob v2.1.0+

| Bottleneck in Stock IBM Bob | Root Cause Found in `bob-code` | Fix Applied by This Repo |
|---|---|---|
| **Constant Approval Popups** | Default `allowed_permissions` is `["read"]` only; `allowedExecutors` only allows 15 read-only commands and matches exact word prefixes (`Bvs`/`Uvs`). | **Safe Turbo Mode:** Enables all 9 permission groups (`read`, `edit`, `execute`, `mcp`, `skill`, `todo`, `subtask`, `subagent`, `mode`), **156** 1-word dev prefixes (`npm`, `node`, `cargo`, `git`, `python`, `go`, `dotnet`, `docker`, `kubectl`, `mkdir`, etc.) + **40** longer-prefix destructive deny rules (`rm -rf`, `git push --force`, `git reset --hard`, `npm publish`, `cargo publish`, `terraform destroy`, `kubectl delete`, `DROP TABLE`, etc.), keeping `isCommandSecurityEnabled: true`. |
| **Cluttered Mode Picker & Broken Subagents** | Dumping dozens of modes into the narrow `w-48` dropdown ruins UX; Bob's `parseAgentFile` (`TFi`) does not strip quotes from `name:`, ignores `tools:`, requires YAML block-list `groups:`, and only reads `<workspace>/.bob/agents/*.md`. | **2-Layer Architecture:** **7 curated executive modes** in `~/.bob/settings/custom_modes.yaml` (or all 23 via `--all-modes`) + **23 PE-AGENTS-IBM specialists** compiled into `<workspace>/.bob/agents/*.md` matching Bob's exact `TFi` parser (`model: premium`, `maxTurns: 50`, `allowForkContext: true`, `## Output Constraints`). |
| **Weak Subagent / Model Tier Fallback** | Built-in `explore` subagent forces `model: explorer`, and production `MODEL_TIERS` collapses `fast/premium/ultra` to `premium-ide` while hiding the UI picker; however `HarnessSession` honors `session.model` in `~/.bob/settings/settings.json`. | Pins **100% of custom subagents to `model: premium`**, instructs the orchestrator to avoid `explore` for complex reasoning, and exposes `set-model <id\|default>` to test/revert direct gateway model overrides safely with automatic SHA-256 backup. |
| **Missing Global MCPs & Auto-Approve** | Global MCPs live in `~/.bob/settings/mcp.json` (not `.vscode/mcp.json`), Bob does **not** expand `${VAR}`, and MCP auto-approval requires per-tool `alwaysAllow`. | Generates `~/.bob/settings/mcp.json` with verified absolute host paths, zero `${VAR}`, and read-only `alwaysAllow` lists: standalone `pe-agents-ibm` MCP (Rust native or zero-deps Node fallback) + auto-detected `rea`, `chrome-devtools`, `gopls`, and `fetch`. |
| **Missing Global Rules & Skills** | `~/.bob/rules/` and `~/.bob/skills/` start empty. | Installs `rules/00-pe-agents-global.md` and syncs **47** curated software engineering, SRE, security, IBM Carbon v11, and reverse-engineering skills to `~/.bob/skills/`. |

---

## Quick Start (1 Command)

### Option 1 — Let Any AI Install It for You
Clone this repository on the target computer and prompt your AI assistant:

> **Prompt:** `Read INSTALL_FOR_AI.md and execute the complete IBM Bob tuning installation on this computer, including compiling .bob/agents for my current project workspace.`

### Option 2 — Direct Terminal Install

**Windows (PowerShell):**
```powershell
git clone https://github.com/eriktonon/ibm-bob-tuning.git
cd ibm-bob-tuning
.\scripts\install.ps1 -Workspace C:\path\to\your\project
```

**macOS / Linux (Bash):**
```bash
git clone https://github.com/eriktonon/ibm-bob-tuning.git
cd ibm-bob-tuning
./scripts/install.sh --workspace /path/to/your/project
```

> **Note:** If `cargo` is installed, the installer compiles and uses the native Rust binary (`target/release/bob-tuner`). If `cargo` is missing (e.g., locked corporate machine), it automatically runs the zero-dependency Node.js engine (`scripts/bob-sync.mjs`) with identical output — zero `npm install` required.

### Final Step Inside IBM Bob IDE
1. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) → **`Developer: Reload Window`** (initializes the newly registered global MCP servers).
2. Open a new chat (`+`) — **`PE-AGENTS-IBM Orquestrador`** is now your default mode.

---

## CLI Commands Reference

Using compiled Rust (`./target/release/bob-tuner`) or Node fallback (`node scripts/bob-sync.mjs`):

```bash
# Audit current IBM Bob configuration
bob-tuner status --workspace /path/to/project

# Apply full tuning (automatic SHA-256 backup in ~/.bob/settings/backups/)
bob-tuner sync --workspace /path/to/project

# Optional: expose all 23 modes in the dropdown instead of the 7 curated executive modes
bob-tuner sync --all-modes --workspace /path/to/project

# Optional: test a direct model override in session.model (reversible in 1 command)
bob-tuner set-model claude-opus-4-6
bob-tuner set-model default   # reverts cleanly to "" (premium-ide)

# Create manual SHA-256 backup
bob-tuner backup

# Validate agents (Bob TFi parser), skills, YAML/JSON & MCP stdio E2E
bob-tuner validate
node scripts/bob-sync.mjs validate
node scripts/bob-sync.mjs dlp-audit
```

---

## Included Specialist Pack (23 Agents & 47 Skills)

- **7 Curated Picker Modes:** `pe-orquestrador`, `pe-arquitetura`, `pe-reverse-engineer`, `pe-seguranca`, `pe-tester`, `pe-sre`, `pe-documentacao`.
- **23 Native `.bob/agents/*.md` Specialists (`model: premium`):** Orchestrator, Architecture, Modernization, Backend Router + Rust / Java / Go / Python / Node.js / .NET specialists, Frontend (React/Next.js + IBM Carbon v11), Blazor WASM, ASP.NET Core MVC, React Native, Database (PostgreSQL), SRE, SRE Investigator, Security (OWASP/CVE), Tester, Documentation, Brain (Context Bank), Research, and Reverse Engineer (REA).
- **47 Technical Skills:** Clean Architecture, Event-Driven Architecture, Build-to-Manage, FinOps SRE, K8s Policy/Troubleshoot, Terraform, Ansible, AWS/Azure/GCP CLI, GitHub Actions, CI/CD, OWASP Review, Security Lib Scan, Reverse Engineer Anything (REA), Spec/Story/Elicitation, Verification Harness, IBM Carbon v11 Tokens & Visual QA, Accessibility (WCAG 2.1 AA), and React/Rust/K8s scaffolds.

---

## Rollback

Every `sync` and `set-model` call creates a timestamped SHA-256 verified snapshot under:
- Windows: `%USERPROFILE%\.bob\settings\backups\<YYYYMMDD_HHMMSS>\`
- macOS/Linux: `~/.bob/settings/backups/<YYYYMMDD_HHMMSS>/`

To restore previous settings, copy `settings.json`, `custom_modes.yaml`, and `mcp.json` from any backup folder back into `~/.bob/settings/` and reload IBM Bob.

## Security & Compliance Note

This repository **never** patches `IBM Bob.exe`, `dist/extension.js`, or enterprise MDM/Registry policies. Every modification uses strictly documented, native IBM Bob user configuration files (`~/.bob/settings/`, `~/.bob/rules/`, `~/.bob/skills/`, `<workspace>/.bob/agents/`).
