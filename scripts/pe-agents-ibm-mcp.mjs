#!/usr/bin/env node
/**
 * Standalone Zero-Dependencies MCP Server (`pe-agents-ibm`) for IBM Bob v2.1.0+
 * Fallback runtime when the Rust `bob-tuner` binary is not compiled on the host.
 * Protocol: JSON-RPC 2.0 over stdio (MCP 2024-11-05).
 */
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoIdx = process.argv.indexOf("--repo");
const REPO_ROOT = path.resolve(
  repoIdx >= 0 && process.argv[repoIdx + 1]
    ? process.argv[repoIdx + 1]
    : process.env.BOB_TUNING_REPO || path.resolve(__dirname, "..")
);

function isSafeIdentifier(name) {
  return (
    typeof name === "string" &&
    name.length > 0 &&
    name.length <= 128 &&
    !name.includes("..") &&
    !name.includes("/") &&
    !name.includes("\\") &&
    /^[a-zA-Z0-9._-]+$/.test(name)
  );
}

const TOOLS = [
  {
    name: "list_agents",
    description: "Lists all PE-AGENTS-IBM specialist agents available in ibm-bob-tuning.",
    inputSchema: { type: "object", properties: {}, additionalProperties: false },
  },
  {
    name: "get_agent",
    description: "Returns the complete specification of a PE-AGENTS-IBM specialist agent.",
    inputSchema: {
      type: "object",
      properties: {
        name: { type: "string", description: "Agent slug (e.g. pe-agents-ibm-rust-backend)" },
      },
      required: ["name"],
      additionalProperties: false,
    },
  },
  {
    name: "list_skills",
    description: "Lists all reusable PE-AGENTS-IBM skills available in ibm-bob-tuning.",
    inputSchema: { type: "object", properties: {}, additionalProperties: false },
  },
  {
    name: "get_skill",
    description: "Returns the full SKILL.md content of a PE-AGENTS-IBM skill.",
    inputSchema: {
      type: "object",
      properties: {
        name: { type: "string", description: "Skill slug (e.g. revisao-owasp, verification-harness)" },
      },
      required: ["name"],
      additionalProperties: false,
    },
  },
  {
    name: "get_tuning_policy",
    description: "Returns the active IBM Bob Turbo Mode allow/deny command policy and curated modes.",
    inputSchema: { type: "object", properties: {}, additionalProperties: false },
  },
  {
    name: "search_knowledge",
    description: "Searches across all local PE-AGENTS-IBM agents, skills, and rules for a keyword or phrase.",
    inputSchema: {
      type: "object",
      properties: {
        query: { type: "string", description: "Keyword or phrase to search" },
      },
      required: ["query"],
      additionalProperties: false,
    },
  },
];

function walkMarkdownFiles(dir, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walkMarkdownFiles(full, out);
    else if (entry.isFile() && entry.name.endsWith(".md")) out.push(full);
  }
  return out;
}

function executeTool(name, args = {}) {
  const agentsDir = path.join(REPO_ROOT, "agents");
  const skillsDir = path.join(REPO_ROOT, "skills");

  switch (name) {
    case "list_agents": {
      const files = fs
        .readdirSync(agentsDir)
        .filter((f) => f.endsWith(".md"))
        .map((f) => f.slice(0, -3))
        .sort();
      return `PE-AGENTS-IBM Specialists (${files.length}):\n` + files.map((f) => `- ${f}`).join("\n");
    }
    case "get_agent": {
      const raw = typeof args.name === "string" ? args.name.trim() : "";
      if (!isSafeIdentifier(raw)) throw new Error("Invalid agent identifier");
      const stem = raw.replace(/\.agent\.md$/, "").replace(/\.md$/, "");
      const target = path.join(agentsDir, `${stem}.md`);
      if (!fs.existsSync(target)) throw new Error(`Agent '${raw}' not found`);
      return fs.readFileSync(target, "utf8");
    }
    case "list_skills": {
      const dirs = fs
        .readdirSync(skillsDir, { withFileTypes: true })
        .filter((d) => d.isDirectory() && fs.existsSync(path.join(skillsDir, d.name, "SKILL.md")))
        .map((d) => d.name)
        .sort();
      return `PE-AGENTS-IBM Skills (${dirs.length}):\n` + dirs.map((d) => `- ${d}`).join("\n");
    }
    case "get_skill": {
      const raw = typeof args.name === "string" ? args.name.trim() : "";
      if (!isSafeIdentifier(raw)) throw new Error("Invalid skill identifier");
      const target = path.join(skillsDir, raw, "SKILL.md");
      if (!fs.existsSync(target)) throw new Error(`Skill '${raw}' not found`);
      return fs.readFileSync(target, "utf8");
    }
    case "get_tuning_policy": {
      return fs.readFileSync(path.join(REPO_ROOT, "config", "turbo-policy.json"), "utf8");
    }
    case "search_knowledge": {
      const q = typeof args.query === "string" ? args.query.trim().toLowerCase() : "";
      if (!q) throw new Error("Missing required argument 'query'");
      const hits = [];
      for (const sub of ["agents", "skills", "rules"]) {
        for (const file of walkMarkdownFiles(path.join(REPO_ROOT, sub))) {
          const lines = fs.readFileSync(file, "utf8").split(/\r?\n/);
          for (let i = 0; i < lines.length; i++) {
            if (lines[i].toLowerCase().includes(q)) {
              hits.push(`${path.relative(REPO_ROOT, file)}:${i + 1}: ${lines[i].trim()}`);
              if (hits.length >= 25) return hits.join("\n");
              break;
            }
          }
        }
      }
      return hits.length ? hits.join("\n") : `No matches found for '${q}'.`;
    }
    default:
      throw new Error(`Unknown MCP tool: '${name}'`);
  }
}

function handleRequest(req) {
  const method = req?.method || "";
  if (req?.id === undefined || req?.id === null || method.startsWith("notifications/")) {
    return null;
  }
  const id = req.id;

  if (method === "initialize") {
    return {
      jsonrpc: "2.0",
      id,
      result: {
        protocolVersion: req.params?.protocolVersion || "2024-11-05",
        capabilities: { tools: { listChanged: false } },
        serverInfo: { name: "pe-agents-ibm", version: "1.0.0-node" },
        instructions: "Standalone PE-AGENTS-IBM specialist catalog, skills, and IBM Bob tuning policy.",
      },
    };
  }
  if (method === "ping") {
    return { jsonrpc: "2.0", id, result: {} };
  }
  if (method === "tools/list") {
    return { jsonrpc: "2.0", id, result: { tools: TOOLS } };
  }
  if (method === "tools/call") {
    const toolName = req.params?.name || "";
    const toolArgs = req.params?.arguments || {};
    try {
      const text = executeTool(toolName, toolArgs);
      return {
        jsonrpc: "2.0",
        id,
        result: { content: [{ type: "text", text }], isError: false },
      };
    } catch (err) {
      return {
        jsonrpc: "2.0",
        id,
        result: { content: [{ type: "text", text: `Error: ${err.message}` }], isError: true },
      };
    }
  }
  return {
    jsonrpc: "2.0",
    id,
    error: { code: -32601, message: `Method not found: ${method}` },
  };
}

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on("line", (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;
  let req;
  try {
    req = JSON.parse(trimmed);
  } catch (err) {
    process.stdout.write(
      JSON.stringify({ jsonrpc: "2.0", id: null, error: { code: -32700, message: `Parse error: ${err.message}` } }) + "\n"
    );
    return;
  }
  const resp = handleRequest(req);
  if (resp) process.stdout.write(JSON.stringify(resp) + "\n");
});
