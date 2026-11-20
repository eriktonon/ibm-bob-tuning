#!/usr/bin/env node
/**
 * Zero-Dependencies Fallback Installer, Validator & DLP Gate for IBM Bob v2.1.0+
 * Exact feature parity with the native Rust `bob-tuner` CLI so installation succeeds
 * even on locked corporate machines without Rust/Cargo or npm registry access.
 */
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const HOME = os.homedir();
const POLICY_PATH = path.join(REPO_ROOT, "config", "turbo-policy.json");
const VALID_GROUPS = new Set([
  "read", "edit", "execute", "mcp", "skill", "todo", "artifact", "subagent", "mode",
]);
const MODEL_TIERS = new Set(["fast", "premium", "ultra", "explorer"]);

function loadPolicy() {
  return JSON.parse(fs.readFileSync(POLICY_PATH, "utf8"));
}

function sanitizeAscii(input) {
  const map = {
    "\u2018": "'", "\u2019": "'", "\u201C": '"', "\u201D": '"',
    "\u2013": "-", "\u2014": "-", "\u00A0": " ", "\u2026": "...", "\u2192": "->",
    "á": "a", "à": "a", "â": "a", "ã": "a", "ä": "a",
    "Á": "A", "À": "A", "Â": "A", "Ã": "A", "Ä": "A",
    "é": "e", "è": "e", "ê": "e", "ë": "e", "É": "E", "È": "E", "Ê": "E", "Ë": "E",
    "í": "i", "ì": "i", "î": "i", "ï": "i", "Í": "I", "Ì": "I", "Î": "I", "Ï": "I",
    "ó": "o", "ò": "o", "ô": "o", "õ": "o", "ö": "o",
    "Ó": "O", "Ò": "O", "Ô": "O", "Õ": "O", "Ö": "O",
    "ú": "u", "ù": "u", "û": "u", "ü": "u", "Ú": "U", "Ù": "U", "Û": "U", "Ü": "U",
    "ç": "c", "Ç": "C", "ñ": "n", "Ñ": "N",
  };
  let out = "";
  for (const ch of String(input || "")) {
    if (ch in map) out += map[ch];
    else if (ch.charCodeAt(0) < 128) out += ch;
  }
  return out;
}

function sha256(buf) {
  return crypto.createHash("sha256").update(buf).digest("hex");
}

function writeAtomic(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const tmp = `${filePath}.tmp.${process.pid}`;
  fs.writeFileSync(tmp, content, "utf8");
  try {
    fs.renameSync(tmp, filePath);
  } catch {
    fs.copyFileSync(tmp, filePath);
    fs.rmSync(tmp, { force: true });
  }
}

function backupSettings() {
  const settingsDir = path.join(HOME, ".bob", "settings");
  const stamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\..+/, "").replace("T", "_");
  const bakDir = path.join(settingsDir, "backups", stamp);
  fs.mkdirSync(bakDir, { recursive: true });
  const hashes = {};
  for (const fname of ["settings.json", "custom_modes.yaml", "mcp.json"]) {
    const src = path.join(settingsDir, fname);
    if (fs.existsSync(src)) {
      const data = fs.readFileSync(src);
      hashes[fname] = sha256(data);
      fs.writeFileSync(path.join(bakDir, fname), data);
    }
  }
  writeAtomic(
    path.join(bakDir, "manifest.sha256.json"),
    JSON.stringify({ created_at_utc: new Date().toISOString(), source_dir: settingsDir, sha256: hashes }, null, 2) + "\n"
  );
  return bakDir;
}

/** Exact replica of IBM Bob v2.1.0 `TFi`/`IFi`/`CFi` frontmatter parser */
function parseBobAgent(content, filename) {
  const normalized = content.replace(/\r\n/g, "\n");
  const m = normalized.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  if (!m) return null;
  const fm = {};
  let currentKey = "";
  for (const line of m[1].split("\n")) {
    const item = line.match(/^\s+-\s+(.+)/);
    if (item && currentKey) {
      if (Array.isArray(fm[currentKey])) fm[currentKey].push(item[1].trim());
      else fm[currentKey] = [item[1].trim()];
      continue;
    }
    const kv = line.match(/^(\w+):\s*(.*)/);
    if (!kv) continue;
    currentKey = kv[1];
    const val = kv[2].trim();
    if (val === "true") fm[currentKey] = true;
    else if (val === "false") fm[currentKey] = false;
    else if (val !== "") fm[currentKey] = val;
  }
  const body = m[2].trim();
  const id = typeof fm.name === "string" ? fm.name : filename.replace(/\.md$/, "");
  return {
    id,
    description: typeof fm.description === "string" ? fm.description : id,
    model: typeof fm.model === "string" && MODEL_TIERS.has(fm.model) ? fm.model : null,
    maxTurns: Number(fm.maxTurns) || 50,
    allowForkContext: fm.allowForkContext !== false,
    groups: Array.isArray(fm.groups) ? fm.groups : null,
    hasOutputConstraints: body.includes("## Output Constraints"),
    body,
    raw: normalized,
  };
}

function deriveSlug(fileStem) {
  const base = fileStem.replace(/^pe-agents-ibm-/, "");
  const short =
    base === "aspnetcore-mvc-frontend"
      ? "mvc-frontend"
      : base === "blazor-wasm-frontend"
        ? "blazor-frontend"
        : base === "react-native-architect"
          ? "react-native"
          : base;
  return `pe-${short}`.toLowerCase().replace(/[^a-z0-9-]/g, "-");
}

function formatDisplayName(slug) {
  const map = {
    sre: "SRE", mvc: "MVC", dotnet: ".NET", nodejs: "Node.js",
    blazor: "Blazor WASM", reverse: "Reverse", engineer: "Engineer (REA)",
  };
  const parts = slug.replace(/^pe-/, "").split("-").map((w) => map[w] || (w[0].toUpperCase() + w.slice(1)));
  return `PE-AGENTS-IBM ${parts.join(" ")}`;
}

function yamlStr(str) {
  return JSON.stringify(sanitizeAscii(str));
}

function summarizeRole(agent, slug) {
  const clean = sanitizeAscii(agent.body);
  const paras = [];
  for (const blk of clean.split(/\n\n+/)) {
    const t = blk.trim();
    if (!t || t.startsWith("#") || t.startsWith("```") || t.startsWith("|") || t.startsWith("---")) continue;
    const one = t.replace(/\s+/g, " ");
    if (one.length >= 30) paras.push(one);
    if (paras.length >= 2) break;
  }
  const base = (paras.length ? paras.join(" ") : sanitizeAscii(agent.description)).slice(0, 1200);
  if (slug === "pe-orquestrador") {
    return `${base} Operational rules in IBM Bob: (1) Delegate specialized work in parallel via spawn_subagent to compiled specialists in .bob/agents/pe-agents-ibm-*.md (all pinned to model: premium - never use generic explore/explorer for architecture, security, or complex coding); (2) Use MCP pe-agents-ibm (list_agents/get_agent/list_skills/get_skill) and rea MCP for reverse engineering; (3) Apply Option A vs Option B analysis and deterministic verification before finishing.`;
  }
  return `${base} Consult your full specialist specification via MCP pe-agents-ibm (get_agent '${agent.id}') when needed and run deterministic verification before completing.`;
}

function buildCustomModesYaml(agents, policy, allModes) {
  let ordered;
  if (allModes) {
    ordered = [...agents].sort((a, b) => {
      const ia = policy.curated_mode_slugs.indexOf(a.slug);
      const ib = policy.curated_mode_slugs.indexOf(b.slug);
      return (ia === -1 ? 999 : ia) - (ib === -1 ? 999 : ib);
    });
  } else {
    ordered = policy.curated_mode_slugs
      .map((s) => agents.find((a) => a.slug === s))
      .filter(Boolean);
  }

  const lines = ["customModes:"];
  for (const a of ordered) {
    let groups = [...(a.groups || ["read", "edit", "execute", "mcp", "skill", "todo"])];
    if (a.slug === "pe-orquestrador") {
      for (const g of ["read", "edit", "execute", "mcp", "skill", "todo", "artifact", "subagent", "mode"]) {
        if (!groups.includes(g)) groups.push(g);
      }
    }
    const when = sanitizeAscii(a.description).slice(0, 320);
    const customInstr =
      a.slug === "pe-orquestrador"
        ? "Orchestrate in waves (Wave 0 architecture/contract -> Wave 1 parallel implementation -> Wave 2 SRE/Security/Tester). Always prefer spawn_subagent with pe-agents-ibm-* specialists (model: premium) instead of the explore (model: explorer) preset. Run deterministic verification before reporting done."
        : "Apply Anti-Sycophancy, compare Option A vs Option B on structural choices, run deterministic verification, and finish with the [RESULTADO] block.";

    lines.push(`  - slug: ${a.slug}`);
    lines.push(`    name: ${yamlStr(formatDisplayName(a.slug))}`);
    lines.push(`    roleDefinition: ${yamlStr(summarizeRole(a, a.slug))}`);
    lines.push(`    whenToUse: ${yamlStr(when)}`);
    lines.push(`    description: ${yamlStr(when.slice(0, 160))}`);
    lines.push(`    customInstructions: ${yamlStr(customInstr)}`);
    lines.push("    groups:");
    for (const g of groups) lines.push(`      - ${g}`);
  }
  return { yaml: lines.join("\n") + "\n", count: ordered.length };
}

function findInPath(candidates) {
  const dirs = (process.env.PATH || "").split(path.delimiter).filter(Boolean);
  for (const dir of dirs) {
    for (const c of candidates) {
      const full = path.join(dir, c);
      if (fs.existsSync(full) && fs.statSync(full).isFile()) return path.resolve(full);
    }
  }
  return null;
}

function findReaScript(nodeBin) {
  const cands = [];
  if (nodeBin) {
    const d = path.dirname(nodeBin);
    cands.push(path.join(d, "node_modules", "rea-agents", "scripts", "rea.mjs"));
    cands.push(path.join(d, "..", "lib", "node_modules", "rea-agents", "scripts", "rea.mjs"));
  }
  cands.push(path.join(HOME, "AppData", "Roaming", "npm", "node_modules", "rea-agents", "scripts", "rea.mjs"));
  cands.push("/usr/local/lib/node_modules/rea-agents/scripts/rea.mjs");
  cands.push("/opt/homebrew/lib/node_modules/rea-agents/scripts/rea.mjs");
  return cands.find((p) => fs.existsSync(p)) || null;
}

function isBrokenNpmFetch(entry) {
  return (
    entry &&
    Array.isArray(entry.args) &&
    entry.args.some((a) => typeof a === "string" && a.includes("@modelcontextprotocol/server-fetch"))
  );
}

function buildMcpJson(existingMcp) {
  const isWin = process.platform === "win32";
  const servers = {};

  if (existingMcp && existingMcp.mcpServers && typeof existingMcp.mcpServers === "object") {
    for (const [k, v] of Object.entries(existingMcp.mcpServers)) {
      if (k === "fetch" && isBrokenNpmFetch(v)) continue;
      servers[k] = v;
    }
  }

  // Prefer compiled Rust binary if present; otherwise use zero-deps Node MCP server
  const binName = isWin ? "bob-tuner.exe" : "bob-tuner";
  const repoRustBin = path.join(REPO_ROOT, "target", "release", binName);
  const cargoRustBin = path.join(HOME, ".cargo", "bin", binName);
  const rustBin = fs.existsSync(repoRustBin)
    ? repoRustBin
    : fs.existsSync(cargoRustBin)
      ? cargoRustBin
      : null;

  if (rustBin) {
    servers["pe-agents-ibm"] = {
      command: rustBin,
      args: ["mcp", "--repo", REPO_ROOT],
      timeout: 60000,
      alwaysAllow: ["list_agents", "get_agent", "list_skills", "get_skill", "get_tuning_policy", "search_knowledge"],
    };
  } else {
    servers["pe-agents-ibm"] = {
      command: process.execPath,
      args: [path.join(REPO_ROOT, "scripts", "pe-agents-ibm-mcp.mjs"), "--repo", REPO_ROOT],
      timeout: 60000,
      alwaysAllow: ["list_agents", "get_agent", "list_skills", "get_skill", "get_tuning_policy", "search_knowledge"],
    };
  }

  const nodeBin = findInPath(isWin ? ["node.exe"] : ["node"]) || process.execPath;
  const npxBin = findInPath(isWin ? ["npx.cmd", "npx.exe"] : ["npx"]);
  const goBin = findInPath(isWin ? ["go.exe"] : ["go"]);
  const reaScript = findReaScript(nodeBin);

  if (reaScript) {
    servers.rea = {
      command: nodeBin,
      args: [reaScript, "mcp"],
      timeout: 120000,
      alwaysAllow: [
        "analyze_javascript_application", "inspect_analysis_view", "inspect_electron_fuses",
        "inspect_artifact", "inspect_pe_resources", "inspect_pe_imports", "inspect_pe_certificates",
        "inspect_native_exports", "inspect_go_binary", "inspect_rust_binary", "inspect_wasm_artifact",
        "inspect_binary_layout", "inspect_sqlite_database", "open_binary", "close_binary",
        "binary_session", "binary_overview", "list_procedures", "search_procedures",
        "list_strings", "search_strings", "analyze_function", "procedure_pseudo_code",
        "trace_application_feature", "get_evidence_bundle", "list_unknowns",
      ],
    };
  }
  if (npxBin) {
    servers["chrome-devtools"] = {
      command: npxBin,
      args: ["-y", "chrome-devtools-mcp@latest"],
      timeout: 60000,
      alwaysAllow: [
        "list_pages", "take_snapshot", "take_screenshot", "list_console_messages",
        "get_console_message", "list_network_requests", "get_network_request",
        "get_css_styles", "lighthouse_audit",
      ],
    };
  }
  if (goBin) {
    servers.gopls = {
      command: goBin,
      args: ["run", "golang.org/x/tools/gopls@latest", "mcp"],
      timeout: 60000,
      alwaysAllow: [
        "go_diagnostics", "go_file_context", "go_package_api",
        "go_search", "go_symbol_references", "go_workspace",
      ],
    };
  }
  return { mcpServers: servers };
}

function loadAgents() {
  const dir = path.join(REPO_ROOT, "agents");
  const files = fs.readdirSync(dir).filter((f) => f.endsWith(".md")).sort();
  return files.map((f) => {
    const p = path.join(dir, f);
    const parsed = parseBobAgent(fs.readFileSync(p, "utf8"), f);
    if (!parsed) throw new Error(`Failed parsing agent frontmatter: ${p}`);
    parsed.slug = deriveSlug(parsed.id);
    parsed.filePath = p;
    return parsed;
  });
}

function copyDirRecursive(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const s = path.join(src, entry.name);
    const d = path.join(dst, entry.name);
    if (entry.isDirectory()) copyDirRecursive(s, d);
    else if (entry.isFile()) fs.copyFileSync(s, d);
  }
}

function syncCommand(opts) {
  const policy = loadPolicy();
  const agents = loadAgents();
  const settingsDir = path.join(HOME, ".bob", "settings");
  const settingsPath = path.join(settingsDir, "settings.json");
  const modesPath = path.join(settingsDir, "custom_modes.yaml");
  const mcpPath = path.join(settingsDir, "mcp.json");
  const ruleSrc = path.join(REPO_ROOT, "rules", "00-pe-agents-global.md");
  const ruleDst = path.join(HOME, ".bob", "rules", "00-pe-agents-global.md");

  const bakDir = opts.dryRun ? null : backupSettings();

  let existing = {};
  if (fs.existsSync(settingsPath)) {
    existing = JSON.parse(fs.readFileSync(settingsPath, "utf8"));
  }
  existing.session = { ...(existing.session || {}), ...policy.session, defaultMode: policy.default_mode };
  if (opts.model !== undefined) {
    const m = opts.model.trim();
    existing.session.model = !m || m.toLowerCase() === "default" || m.toLowerCase() === "premium-ide" ? "" : m;
  } else if (typeof existing.session.model !== "string") {
    existing.session.model = "";
  }
  existing.chat = { ...(existing.chat || {}), ...policy.chat };
  if (!opts.noTurbo) {
    existing.approval = {
      ...(existing.approval || {}),
      ...policy.approval,
      allowedExecutors: [
        {
          toolId: "execute_command",
          approvedCommands: policy.approved_commands,
          deniedCommands: policy.denied_commands,
        },
      ],
    };
  }
  existing.isCommandSecurityEnabled = policy.isCommandSecurityEnabled;

  const { yaml, count: modesCount } = buildCustomModesYaml(agents, policy, opts.allModes);
  const existingMcp = fs.existsSync(mcpPath) ? JSON.parse(fs.readFileSync(mcpPath, "utf8")) : {};
  const mcpObj = buildMcpJson(existingMcp);
  const mcpText = JSON.stringify(mcpObj, null, 2) + "\n";
  if (mcpText.includes("${")) throw new Error("mcp.json must not contain ${VAR} templates");

  let skillsSynced = 0;
  const srcSkills = path.join(REPO_ROOT, "skills");
  const dstSkills = path.join(HOME, ".bob", "skills");
  if (fs.existsSync(srcSkills)) {
    for (const d of fs.readdirSync(srcSkills, { withFileTypes: true })) {
      if (!d.isDirectory() || !/^[a-z0-9]+(-[a-z0-9]+)*$/.test(d.name)) continue;
      if (!fs.existsSync(path.join(srcSkills, d.name, "SKILL.md"))) continue;
      skillsSynced++;
      if (!opts.dryRun) copyDirRecursive(path.join(srcSkills, d.name), path.join(dstSkills, d.name));
    }
  }

  if (!opts.dryRun) {
    writeAtomic(settingsPath, JSON.stringify(existing, null, 2) + "\n");
    writeAtomic(modesPath, yaml);
    if (fs.existsSync(ruleSrc)) writeAtomic(ruleDst, fs.readFileSync(ruleSrc, "utf8"));
    writeAtomic(mcpPath, mcpText);
    writeAtomic(path.join(settingsDir, ".repo-root"), REPO_ROOT + "\n");
  }

  const wsSummary = [];
  for (const ws of opts.workspaces) {
    const absWs = path.resolve(ws);
    if (!fs.existsSync(absWs)) continue;
    let legacy = 0;
    if (!opts.dryRun) {
      const target = path.join(absWs, ".bob", "agents");
      fs.mkdirSync(target, { recursive: true });
      const legacyFiles = fs.readdirSync(target).filter((f) => f.endsWith(".agent.md"));
      if (legacyFiles.length) {
        const bak = path.join(absWs, ".bob", "agents-legacy-backup");
        fs.mkdirSync(bak, { recursive: true });
        for (const lf of legacyFiles) {
          fs.copyFileSync(path.join(target, lf), path.join(bak, lf));
          fs.rmSync(path.join(target, lf), { force: true });
          legacy++;
        }
      }
      for (const a of agents) {
        writeAtomic(path.join(target, `${a.id}.md`), a.raw);
      }
    }
    wsSummary.push({ ws: absWs, count: agents.length, legacy });
  }

  console.log("================================================================================");
  console.log(`IBM BOB v2.1.0 — PE-AGENTS-IBM SYNC (Node Zero-Deps) [${opts.dryRun ? "DRY-RUN" : "APPLIED"}]`);
  console.log("================================================================================");
  if (bakDir) console.log(`  Backup SHA-256:    ${bakDir}`);
  console.log(`  Turbo Mode:        ${opts.noTurbo ? "DISABLED" : "ENABLED"} (${policy.approved_commands.length} allow / ${policy.denied_commands.length} deny)`);
  console.log(`  session.model:     ${existing.session.model || '"" (default: premium-ide; subagents: model: premium)'}`);
  console.log(`  Modes in Picker:   ${modesCount} -> ${modesPath}`);
  console.log(`  Global Rules:      ${ruleDst}`);
  console.log(`  Global Skills:     ${skillsSynced} synced to ~/.bob/skills/`);
  console.log(`  Global MCPs (${Object.keys(mcpObj.mcpServers).length}):   [${Object.keys(mcpObj.mcpServers).join(", ")}] -> ${mcpPath}`);
  for (const item of wsSummary) {
    console.log(`  Workspace Agents:  ${item.count} compiled in ${path.join(item.ws, ".bob", "agents")} (legacy archived: ${item.legacy})`);
  }
  console.log("================================================================================");
}

function validateCommand() {
  const policy = loadPolicy();
  const agents = loadAgents();
  if (!agents.length) throw new Error("No agents found");

  for (const a of agents) {
    if (a.id.includes('"') || a.id.includes(" ")) throw new Error(`Invalid agent ID: ${a.id}`);
    if (a.model !== "premium") throw new Error(`Agent ${a.id} model must be 'premium', got '${a.model}'`);
    if (!Array.isArray(a.groups) || !a.groups.includes("read")) throw new Error(`Agent ${a.id} missing valid groups block-list`);
    for (const g of a.groups) {
      if (!VALID_GROUPS.has(g)) throw new Error(`Agent ${a.id} has invalid Bob group '${g}'`);
    }
    if (!a.hasOutputConstraints) throw new Error(`Agent ${a.id} missing ## Output Constraints`);
  }

  const { yaml, count } = buildCustomModesYaml(agents, policy, false);
  if (/[^\x00-\x7F]/.test(yaml)) throw new Error("custom_modes.yaml must be pure ASCII");

  const skillsDir = path.join(REPO_ROOT, "skills");
  let skillsOk = 0;
  for (const d of fs.readdirSync(skillsDir, { withFileTypes: true })) {
    if (!d.isDirectory()) continue;
    if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(d.name) || d.name.length > 64) {
      throw new Error(`Invalid Bob skill name: ${d.name}`);
    }
    if (!fs.existsSync(path.join(skillsDir, d.name, "SKILL.md"))) {
      throw new Error(`Missing SKILL.md in skills/${d.name}`);
    }
    skillsOk++;
  }

  // Live E2E test of Node MCP server
  const payload = [
    JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2024-11-05" } }),
    JSON.stringify({ jsonrpc: "2.0", id: 2, method: "tools/list" }),
    JSON.stringify({ jsonrpc: "2.0", id: 3, method: "tools/call", params: { name: "list_agents", arguments: {} } }),
    JSON.stringify({ jsonrpc: "2.0", id: 4, method: "tools/call", params: { name: "get_agent", arguments: { name: "../../Cargo.toml" } } }),
  ].join("\n") + "\n";

  const r = spawnSync(process.execPath, [path.join(REPO_ROOT, "scripts", "pe-agents-ibm-mcp.mjs"), "--repo", REPO_ROOT], {
    input: payload,
    encoding: "utf8",
    timeout: 10000,
  });
  if (r.status !== 0) throw new Error(`MCP E2E failed: ${r.stderr}`);
  const replies = r.stdout.trim().split(/\r?\n/).map((l) => JSON.parse(l));
  if (replies.length !== 4 || !replies[3].result.isError) {
    throw new Error("MCP E2E path-traversal guard did not reject ../../Cargo.toml");
  }

  console.log(`[PASS] validate: ${agents.length} agents (TFi OK), ${count} curated modes (ASCII OK), ${skillsOk} skills (Bob regex OK), Node MCP E2E OK`);
}

function dlpAuditCommand() {
  const forbidden = [
    { name: "Private LAN IP (192.168.x.x)", rx: /\b192\.168\.\d{1,3}\.\d{1,3}\b/g },
    { name: "Personal Windows path", rx: /C:\\Users\\erikt/gi },
    { name: "Client name (Bradesco/BraDS/HCOM)", rx: /\b(bradesco|brads|hcom|static\.bradesco|dojo@biz)\b/gi },
    { name: "Client bank (BTG/Santander/Gazin)", rx: /\b(btg\s+pactual|santander|gazin)\b/gi },
    { name: "Homelab internal tooling", rx: /\b(opnsense|pfsense|proxmox|tonon-vault|oniontalk|losgallos|jarvis-hud|homelab\.exe|homelab_knowledge|pentest-strix)\b/gi },
    { name: "Other AI IDEs/Agents", rx: /\b(antigravity|claude\s+code|cursor\s+ai|kilo-?code|github\s+copilot|\.cursorrules|\.clauderc)\b/gi },
    { name: "Secret token pattern", rx: /\b(ghp_[A-Za-z0-9]{20,}|gho_[A-Za-z0-9]{20,}|glpat-[A-Za-z0-9_-]{16,}|sk-[A-Za-z0-9]{24,})\b/g },
  ];

  const violations = [];
  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if ([".git", "target", "node_modules"].includes(entry.name)) continue;
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.isFile()) {
        // Skip this script's own regex definitions
        if (path.resolve(full) === path.resolve(fileURLToPath(import.meta.url))) continue;
        const txt = fs.readFileSync(full, "utf8");
        const lines = txt.split(/\r?\n/);
        for (let i = 0; i < lines.length; i++) {
          for (const rule of forbidden) {
            rule.rx.lastIndex = 0;
            const m = rule.rx.exec(lines[i]);
            if (m) {
              violations.push(`${path.relative(REPO_ROOT, full)}:${i + 1} [${rule.name}] -> ${m[0]}`);
            }
          }
        }
      }
    }
  }
  walk(REPO_ROOT);

  if (violations.length) {
    console.error(`[FAIL] DLP Audit found ${violations.length} violation(s):`);
    for (const v of violations.slice(0, 50)) console.error("  - " + v);
    process.exit(1);
  }
  console.log("[PASS] DLP Audit: 0 leaks (0 private IPs, 0 client mentions, 0 homelab refs, 0 other-IDE clutter, 0 secrets).");
}

function parseArgs(argv) {
  const cmd = argv[2] || "status";
  const opts = { workspaces: [], allModes: false, noTurbo: false, dryRun: false, model: undefined };
  for (let i = 3; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--workspace" && argv[i + 1]) opts.workspaces.push(argv[++i]);
    else if (a === "--all-modes") opts.allModes = true;
    else if (a === "--no-turbo") opts.noTurbo = true;
    else if (a === "--dry-run") opts.dryRun = true;
    else if (a === "--model" && argv[i + 1]) opts.model = argv[++i];
    else if (cmd === "set-model" && opts.model === undefined) opts.model = a;
  }
  return { cmd, opts };
}

const { cmd, opts } = parseArgs(process.argv);
if (cmd === "sync") syncCommand(opts);
else if (cmd === "validate") validateCommand();
else if (cmd === "dlp-audit") dlpAuditCommand();
else if (cmd === "backup") console.log("[OK] SHA-256 backup saved at:", backupSettings());
else if (cmd === "set-model") {
  const bak = backupSettings();
  const sp = path.join(HOME, ".bob", "settings", "settings.json");
  const obj = fs.existsSync(sp) ? JSON.parse(fs.readFileSync(sp, "utf8")) : {};
  const raw = (opts.model || "").trim();
  const norm = !raw || raw.toLowerCase() === "default" || raw.toLowerCase() === "premium-ide" ? "" : raw;
  obj.session = { ...(obj.session || {}), model: norm };
  writeAtomic(sp, JSON.stringify(obj, null, 2) + "\n");
  console.log(`[OK] session.model updated to ${norm || '"" (default: premium-ide)'} (backup: ${bak})`);
} else if (cmd === "status") {
  const sp = path.join(HOME, ".bob", "settings", "settings.json");
  const mp = path.join(HOME, ".bob", "settings", "mcp.json");
  const cm = path.join(HOME, ".bob", "settings", "custom_modes.yaml");
  console.log("IBM BOB STATUS:", {
    settings: fs.existsSync(sp) ? sp : "MISSING",
    custom_modes: fs.existsSync(cm) ? cm : "MISSING",
    mcp: fs.existsSync(mp) ? Object.keys(JSON.parse(fs.readFileSync(mp, "utf8")).mcpServers || {}) : "MISSING",
  });
} else {
  console.error(`Unknown command: ${cmd}`);
  process.exit(2);
}
