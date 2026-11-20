use chrono::Utc;
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const EMBEDDED_POLICY_JSON: &str = include_str!("../config/turbo-policy.json");

const VALID_BOB_GROUPS: &[&str] = &[
    "read", "edit", "execute", "mcp", "skill", "todo", "artifact", "subagent", "mode",
];

#[derive(Parser, Debug)]
#[command(
    name = "bob-tuner",
    version = "1.0.0",
    about = "Native tuning, PE-AGENTS-IBM agent compiler, safe Turbo Mode & MCP server for IBM Bob v2.1.0+"
)]
struct Cli {
    /// Path to the ibm-bob-tuning repository root (auto-detected if omitted)
    #[arg(long, global = true)]
    repo: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print diagnostic audit of IBM Bob configuration, modes, MCPs, skills, rules, and workspace agents
    Status {
        /// Additional workspaces to inspect for `.bob/agents/*.md`
        #[arg(long)]
        workspace: Vec<PathBuf>,
    },
    /// Create a SHA-256 verified backup of `~/.bob/settings/`
    Backup,
    /// Apply full IBM Bob tuning (safe Turbo Mode, curated modes, native agents, skills, rules, and MCPs)
    Sync {
        /// Target workspaces to compile `.bob/agents/*.md` into (defaults to current directory if not this repo, plus any provided)
        #[arg(long)]
        workspace: Vec<PathBuf>,
        /// Expose all agents in the Bob mode dropdown instead of the 7 curated executive modes
        #[arg(long, default_value_t = false)]
        all_modes: bool,
        /// Optional `session.model` override (pass `default` or `""` to reset to `premium-ide`)
        #[arg(long)]
        model: Option<String>,
        /// Skip writing Turbo Mode auto-approval rules into `settings.json`
        #[arg(long, default_value_t = false)]
        no_turbo: bool,
        /// Simulate changes without writing files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Set or reset `session.model` in `~/.bob/settings/settings.json` (with automatic backup)
    SetModel {
        /// Target model ID (e.g. `claude-opus-4-6`, `claude-sonnet-4-6`, or `default` for `premium-ide`)
        model: String,
    },
    /// Validate repository policy, agents (Bob TFi parser parity), skills, and rules
    Validate,
    /// Run the standalone JSON-RPC 2.0 stdio MCP server (`pe-agents-ibm`)
    Mcp,
}

#[derive(Debug, Clone, Deserialize)]
struct TurboPolicy {
    default_mode: String,
    curated_mode_slugs: Vec<String>,
    session: Map<String, Value>,
    chat: Map<String, Value>,
    approval: Map<String, Value>,
    #[serde(rename = "isCommandSecurityEnabled")]
    is_command_security_enabled: bool,
    approved_commands: Vec<String>,
    denied_commands: Vec<String>,
}

#[derive(Debug, Clone)]
struct BobAgent {
    file_stem: String,
    bob_slug: String,
    display_name: String,
    description: String,
    groups: Vec<String>,
    model: String,
    max_turns: u32,
    allow_fork_context: bool,
    body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BobCustomModesFile {
    #[serde(rename = "customModes")]
    custom_modes: Vec<BobCustomModeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BobCustomModeEntry {
    slug: String,
    name: String,
    #[serde(rename = "roleDefinition")]
    role_definition: String,
    #[serde(rename = "whenToUse", skip_serializing_if = "Option::is_none")]
    when_to_use: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "customInstructions", skip_serializing_if = "Option::is_none")]
    custom_instructions: Option<String>,
    groups: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repo_root = resolve_repo_root(cli.repo.as_deref()).map_err(std::io::Error::other)?;
    let home = resolve_home_dir().map_err(std::io::Error::other)?;
    let policy = load_policy(&repo_root).map_err(std::io::Error::other)?;

    match cli.command {
        Commands::Status { workspace } => {
            print_status(&home, &workspace).map_err(std::io::Error::other)?;
        }
        Commands::Backup => {
            let bak = backup_bob_settings(&home).map_err(std::io::Error::other)?;
            println!("[OK] SHA-256 backup saved at: {}", bak.display());
        }
        Commands::Sync {
            workspace,
            all_modes,
            model,
            no_turbo,
            dry_run,
        } => {
            sync_bob(
                &repo_root,
                &home,
                &policy,
                &workspace,
                all_modes,
                model.as_deref(),
                !no_turbo,
                dry_run,
            )
            .map_err(std::io::Error::other)?;
        }
        Commands::SetModel { model } => {
            let normalized = normalize_model_arg(&model);
            let bak = backup_bob_settings(&home).map_err(std::io::Error::other)?;
            let settings_path = home.join(".bob").join("settings").join("settings.json");
            let mut root = read_json_object_or_empty(&settings_path).map_err(std::io::Error::other)?;
            ensure_object_entry(&mut root, "session")
                .insert("model".to_string(), Value::String(normalized.clone()));
            write_json_atomic(&settings_path, &Value::Object(root)).map_err(std::io::Error::other)?;
            let label = if normalized.is_empty() {
                "\"\" (default: premium-ide)"
            } else {
                normalized.as_str()
            };
            println!(
                "[OK] session.model updated to {} (backup: {})",
                label,
                bak.display()
            );
        }
        Commands::Validate => {
            validate_repo(&repo_root, &policy).map_err(std::io::Error::other)?;
        }
        Commands::Mcp => {
            run_mcp_stdio_server(&repo_root).map_err(std::io::Error::other)?;
        }
    }

    Ok(())
}

fn canonicalize_clean(path: &Path) -> PathBuf {
    let resolved = fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    let s = resolved.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        resolved
    }
}

fn resolve_home_dir() -> Result<PathBuf, String> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Could not resolve HOME / USERPROFILE environment variable".to_string())
}

fn resolve_repo_root(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        return Ok(canonicalize_clean(p));
    }
    if let Ok(env_p) = std::env::var("BOB_TUNING_REPO") {
        let p = PathBuf::from(env_p.trim());
        if p.join("agents").is_dir() {
            return Ok(canonicalize_clean(&p));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("agents").is_dir() && cwd.join("config").join("turbo-policy.json").is_file() {
            return Ok(canonicalize_clean(&cwd));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        while let Some(dir) = cur {
            if dir.join("agents").is_dir() && dir.join("config").join("turbo-policy.json").is_file()
            {
                return Ok(canonicalize_clean(dir));
            }
            cur = dir.parent();
        }
    }
    Err("Could not locate ibm-bob-tuning repository root. Pass --repo <path> or set BOB_TUNING_REPO.".to_string())
}

fn load_policy(repo_root: &Path) -> Result<TurboPolicy, String> {
    let disk_path = repo_root.join("config").join("turbo-policy.json");
    let raw = if disk_path.is_file() {
        fs::read_to_string(&disk_path)
            .map_err(|e| format!("Failed reading '{}': {}", disk_path.display(), e))?
    } else {
        EMBEDDED_POLICY_JSON.to_string()
    };
    serde_json::from_str(&raw).map_err(|e| format!("Invalid turbo-policy.json: {}", e))
}

fn normalize_model_arg(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("default") || t.eq_ignore_ascii_case("premium-ide") {
        String::new()
    } else {
        t.to_string()
    }
}

fn sanitize_ascii(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201C}' | '\u{201D}' => out.push('"'),
            '\u{2013}' | '\u{2014}' => out.push('-'),
            '\u{00A0}' => out.push(' '),
            '\u{2026}' => out.push_str("..."),
            '\u{2192}' => out.push_str("->"),
            'á' | 'à' | 'â' | 'ã' | 'ä' => out.push('a'),
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => out.push('A'),
            'é' | 'è' | 'ê' | 'ë' => out.push('e'),
            'É' | 'È' | 'Ê' | 'Ë' => out.push('E'),
            'í' | 'ì' | 'î' | 'ï' => out.push('i'),
            'Í' | 'Ì' | 'Î' | 'Ï' => out.push('I'),
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => out.push('o'),
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => out.push('O'),
            'ú' | 'ù' | 'û' | 'ü' => out.push('u'),
            'Ú' | 'Ù' | 'Û' | 'Ü' => out.push('U'),
            'ç' => out.push('c'),
            'Ç' => out.push('C'),
            'ñ' => out.push('n'),
            'Ñ' => out.push('N'),
            c if c.is_ascii() => out.push(c),
            _ => {}
        }
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed creating dir '{}': {}", parent.display(), e))?;
    }
    let tmp_name = format!(
        "{}.tmp.{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("bob"),
        std::process::id()
    );
    let tmp_path = path.with_file_name(tmp_name);
    fs::write(&tmp_path, content.as_bytes())
        .map_err(|e| format!("Failed writing temp '{}': {}", tmp_path.display(), e))?;
    if path.exists() {
        if fs::rename(&tmp_path, path).is_err() {
            fs::copy(&tmp_path, path).map_err(|e| {
                let _ = fs::remove_file(&tmp_path);
                format!("Failed replacing '{}': {}", path.display(), e)
            })?;
            let _ = fs::remove_file(&tmp_path);
        }
    } else {
        fs::rename(&tmp_path, path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Failed moving temp to '{}': {}", path.display(), e)
        })?;
    }
    Ok(())
}

fn write_json_atomic(path: &Path, val: &Value) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(val)
        .map_err(|e| format!("Failed serializing JSON: {}", e))?;
    write_text_atomic(path, &format!("{}\n", serialized))
}

fn read_json_object_or_empty(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed reading '{}': {}", path.display(), e))?;
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON in '{}': {}", path.display(), e))?;
    parsed
        .as_object()
        .cloned()
        .ok_or_else(|| format!("Root of '{}' is not a JSON object", path.display()))
}

fn ensure_object_entry<'a>(
    root: &'a mut Map<String, Value>,
    key: &str,
) -> &'a mut Map<String, Value> {
    let needs_reset = !root.get(key).map(|v| v.is_object()).unwrap_or(false);
    if needs_reset {
        root.insert(key.to_string(), Value::Object(Map::new()));
    }
    match root.get_mut(key) {
        Some(Value::Object(map)) => map,
        _ => unreachable!("key was just inserted as Value::Object"),
    }
}

fn backup_bob_settings(home: &Path) -> Result<PathBuf, String> {
    let settings_dir = home.join(".bob").join("settings");
    let ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = settings_dir.join("backups").join(&ts);
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed creating backup dir '{}': {}", backup_dir.display(), e))?;

    let mut manifest = BTreeMap::new();
    for fname in ["settings.json", "custom_modes.yaml", "mcp.json"] {
        let src = settings_dir.join(fname);
        if src.is_file() {
            let bytes = fs::read(&src)
                .map_err(|e| format!("Failed reading '{}': {}", src.display(), e))?;
            manifest.insert(fname.to_string(), sha256_hex(&bytes));
            fs::write(backup_dir.join(fname), &bytes)
                .map_err(|e| format!("Failed writing backup '{}': {}", fname, e))?;
        }
    }
    let manifest_json = json!({
        "created_at_utc": Utc::now().to_rfc3339(),
        "source_dir": settings_dir.display().to_string(),
        "sha256": manifest
    });
    write_json_atomic(&backup_dir.join("manifest.sha256.json"), &manifest_json)?;
    Ok(backup_dir)
}

fn derive_bob_slug(file_stem: &str) -> String {
    let base = file_stem
        .strip_prefix("pe-agents-ibm-")
        .unwrap_or(file_stem);
    let short = match base {
        "aspnetcore-mvc-frontend" => "mvc-frontend",
        "blazor-wasm-frontend" => "blazor-frontend",
        "react-native-architect" => "react-native",
        other => other,
    };
    format!("pe-{}", short)
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    let normalized = raw.replace("\r\n", "\n");
    let rest = normalized.strip_prefix("---\n")?;
    let end_idx = rest.find("\n---\n")?;
    Some((
        rest[..end_idx].to_string(),
        rest[end_idx + 5..].trim().to_string(),
    ))
}

/// Parses an agent file using the exact same scalar/block-list grammar as IBM Bob's `TFi`/`IFi`
fn parse_bob_agent_file(path: &Path) -> Result<BobAgent, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed reading agent '{}': {}", path.display(), e))?;
    let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;
    let file_stem = fname.strip_suffix(".md").unwrap_or(fname).to_string();

    let (fm, body) = split_frontmatter(&raw)
        .ok_or_else(|| format!("Missing YAML frontmatter in '{}'", path.display()))?;

    let mut scalars: BTreeMap<String, String> = BTreeMap::new();
    let mut lists: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current_key = String::new();

    for line in fm.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("- ") {
            if line.starts_with(' ') && !current_key.is_empty() {
                lists
                    .entry(current_key.clone())
                    .or_default()
                    .push(rest.trim().to_string());
                continue;
            }
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim();
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                current_key = key.to_string();
                let val = v.trim();
                if !val.is_empty() {
                    scalars.insert(current_key.clone(), val.to_string());
                }
            }
        }
    }

    let name = scalars
        .get("name")
        .cloned()
        .unwrap_or_else(|| file_stem.clone());
    if name.contains('"') || name.contains('\'') || name.contains(' ') {
        return Err(format!(
            "Agent '{}' has invalid unquoted slug name '{}' (Bob TFi parser does not strip quotes)",
            path.display(),
            name
        ));
    }

    let description = scalars
        .get("description")
        .cloned()
        .unwrap_or_else(|| name.clone());
    let model = scalars
        .get("model")
        .cloned()
        .unwrap_or_else(|| "premium".to_string());
    let max_turns = scalars
        .get("maxTurns")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(50);
    let allow_fork_context = scalars
        .get("allowForkContext")
        .map(|v| v == "true")
        .unwrap_or(true);
    let groups = lists.get("groups").cloned().unwrap_or_else(|| {
        vec![
            "read".into(),
            "edit".into(),
            "execute".into(),
            "mcp".into(),
            "skill".into(),
            "todo".into(),
        ]
    });

    let bob_slug = derive_bob_slug(&name);
    let display_name = scalars
        .get("displayName")
        .cloned()
        .unwrap_or_else(|| format_display_name(&bob_slug));

    Ok(BobAgent {
        file_stem: name,
        bob_slug,
        display_name,
        description,
        groups,
        model,
        max_turns,
        allow_fork_context,
        body,
    })
}

fn format_display_name(bob_slug: &str) -> String {
    let suffix = bob_slug.strip_prefix("pe-").unwrap_or(bob_slug);
    let title: String = suffix
        .split('-')
        .map(|w| match w {
            "sre" => "SRE".to_string(),
            "mvc" => "MVC".to_string(),
            "dotnet" => ".NET".to_string(),
            "nodejs" => "Node.js".to_string(),
            "blazor" => "Blazor WASM".to_string(),
            "reverse" => "Reverse".to_string(),
            "engineer" => "Engineer (REA)".to_string(),
            other => {
                let mut chars = other.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!("PE-AGENTS-IBM {}", title)
}

fn load_repo_agents(repo_root: &Path) -> Result<Vec<BobAgent>, String> {
    let dir = repo_root.join("agents");
    if !dir.is_dir() {
        return Err(format!("Agents directory not found: {}", dir.display()));
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("Failed listing '{}': {}", dir.display(), e))?
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".md"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();

    let mut agents = Vec::with_capacity(files.len());
    for f in files {
        agents.push(parse_bob_agent_file(&f)?);
    }
    Ok(agents)
}

fn render_bob_agent_md(agent: &BobAgent) -> String {
    let clean_desc = sanitize_ascii(&agent.description)
        .replace('"', "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let truncated_desc: String = clean_desc.chars().take(280).collect();

    let mut groups_yaml = String::new();
    for g in &agent.groups {
        if g == "subagent" || g == "mode" {
            continue;
        }
        if VALID_BOB_GROUPS.contains(&g.as_str()) {
            groups_yaml.push_str(&format!("  - {}\n", g));
        }
    }
    if groups_yaml.is_empty() {
        groups_yaml.push_str("  - read\n  - mcp\n  - skill\n");
    }

    let body = agent.body.trim();
    let has_constraints = body.contains("## Output Constraints");
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", agent.file_stem));
    out.push_str(&format!("description: {}\n", truncated_desc));
    out.push_str(&format!("model: {}\n", agent.model));
    out.push_str(&format!("maxTurns: {}\n", agent.max_turns));
    out.push_str(&format!("allowForkContext: {}\n", agent.allow_fork_context));
    out.push_str("groups:\n");
    out.push_str(&groups_yaml);
    out.push_str("---\n\n");
    out.push_str(body);
    out.push('\n');

    if !has_constraints {
        out.push_str("\n## Output Constraints\n");
        out.push_str("- Reply in the user's language with critical engineering rigor (Anti-Sycophancy) and verifiable evidence.\n");
        out.push_str("- Never claim compilation, tests, or runtime results that were not executed.\n");
        out.push_str("- When invoked as a subagent specialist, end with the [RESULTADO] block containing STATUS, ARQUIVOS_CRIADOS, ARQUIVOS_MODIFICADOS, DECISOES, PENDENCIAS, and PROXIMO_AGENTE.\n");
    }
    out
}

fn summarize_role_definition(agent: &BobAgent) -> String {
    let clean_body = sanitize_ascii(&agent.body);
    let mut paragraphs = Vec::new();
    for block in clean_body.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("```")
            || trimmed.starts_with('|')
            || trimmed.starts_with("---")
        {
            continue;
        }
        let one_line = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        if one_line.len() >= 30 {
            paragraphs.push(one_line);
        }
        if paragraphs.len() >= 2 {
            break;
        }
    }
    let summary = if paragraphs.is_empty() {
        sanitize_ascii(&agent.description)
    } else {
        paragraphs.join(" ")
    };
    let capped: String = summary.chars().take(1200).collect();

    if agent.bob_slug == "pe-orquestrador" {
        format!(
            "{} Operational rules in IBM Bob: (1) Delegate specialized work in parallel via spawn_subagent to the compiled specialists in .bob/agents/pe-agents-ibm-*.md (all pinned to model: premium - never use the generic explore/explorer subagent for architecture, security, or complex coding); (2) Use the pe-agents-ibm MCP server (list_agents/get_agent/list_skills/get_skill) and rea MCP server for reverse engineering; (3) Apply critical Option A vs Option B analysis and deterministic verification before finishing.",
            capped
        )
    } else {
        format!(
            "{} Consult your full specialist specification via MCP pe-agents-ibm (get_agent `{}`) when needed and always run deterministic verification before completing.",
            capped, agent.file_stem
        )
    }
}

fn build_custom_modes_yaml(
    agents: &[BobAgent],
    policy: &TurboPolicy,
    all_modes: bool,
) -> Result<(String, usize), String> {
    let slug_re = Regex::new(r"^[a-zA-Z0-9-]+$")
        .map_err(|e| format!("Invalid slug regex: {}", e))?;

    let ordered: Vec<&BobAgent> = if all_modes {
        let mut list: Vec<&BobAgent> = agents.iter().collect();
        list.sort_by_key(|a| {
            policy
                .curated_mode_slugs
                .iter()
                .position(|s| s == &a.bob_slug)
                .unwrap_or(999)
        });
        list
    } else {
        let mut list = Vec::new();
        for slug in &policy.curated_mode_slugs {
            if let Some(found) = agents.iter().find(|a| &a.bob_slug == slug) {
                list.push(found);
            }
        }
        list
    };

    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    for agent in ordered {
        if !slug_re.is_match(&agent.bob_slug) {
            return Err(format!("Invalid Bob mode slug: '{}'", agent.bob_slug));
        }
        if !seen.insert(agent.bob_slug.clone()) {
            return Err(format!("Duplicate Bob mode slug: '{}'", agent.bob_slug));
        }

        let mut mode_groups = agent.groups.clone();
        if agent.bob_slug == "pe-orquestrador" {
            for g in ["read", "edit", "execute", "mcp", "skill", "todo", "artifact", "subagent", "mode"] {
                if !mode_groups.iter().any(|x| x == g) {
                    mode_groups.push(g.to_string());
                }
            }
        }

        let when = sanitize_ascii(&agent.description);
        let custom_instructions = if agent.bob_slug == "pe-orquestrador" {
            Some(
                "Orchestrate in waves (Wave 0 architecture/contract -> Wave 1 parallel implementation -> Wave 2 SRE/Security/Tester). Always prefer spawn_subagent with pe-agents-ibm-* specialists (model: premium) instead of the explore (model: explorer) preset. Run deterministic verification before reporting done.".to_string(),
            )
        } else {
            Some(
                "Apply Anti-Sycophancy, compare Option A vs Option B on structural choices, run deterministic verification, and finish with the [RESULTADO] block.".to_string(),
            )
        };

        entries.push(BobCustomModeEntry {
            slug: agent.bob_slug.clone(),
            name: sanitize_ascii(&agent.display_name),
            role_definition: summarize_role_definition(agent),
            when_to_use: Some(when.chars().take(320).collect()),
            description: Some(when.chars().take(160).collect()),
            custom_instructions,
            groups: mode_groups,
        });
    }

    let doc = BobCustomModesFile {
        custom_modes: entries,
    };
    let count = doc.custom_modes.len();
    let yaml = serde_yaml::to_string(&doc)
        .map_err(|e| format!("Failed serializing custom_modes.yaml: {}", e))?;
    Ok((yaml, count))
}

fn build_settings_json(
    existing: &Map<String, Value>,
    policy: &TurboPolicy,
    model_override: Option<&str>,
    turbo_enabled: bool,
) -> (Value, String) {
    let mut root = existing.clone();

    let session = ensure_object_entry(&mut root, "session");
    session.insert(
        "defaultMode".to_string(),
        Value::String(policy.default_mode.clone()),
    );
    let effective_model = if let Some(m) = model_override {
        let norm = normalize_model_arg(m);
        session.insert("model".to_string(), Value::String(norm.clone()));
        norm
    } else {
        session
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    for (k, v) in &policy.session {
        session.insert(k.clone(), v.clone());
    }

    let chat = ensure_object_entry(&mut root, "chat");
    for (k, v) in &policy.chat {
        chat.insert(k.clone(), v.clone());
    }

    if turbo_enabled {
        let approval = ensure_object_entry(&mut root, "approval");
        for (k, v) in &policy.approval {
            approval.insert(k.clone(), v.clone());
        }
        approval.insert(
            "allowedExecutors".to_string(),
            json!([
                {
                    "toolId": "execute_command",
                    "approvedCommands": policy.approved_commands,
                    "deniedCommands": policy.denied_commands
                }
            ]),
        );
    }

    root.insert(
        "isCommandSecurityEnabled".to_string(),
        json!(policy.is_command_security_enabled),
    );

    (Value::Object(root), effective_model)
}

fn find_in_path(candidates: &[&str]) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for cand in candidates {
            let full = dir.join(cand);
            if full.is_file() {
                return Some(canonicalize_clean(&full));
            }
        }
    }
    None
}

fn find_rea_script(home: &Path, node_path: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(node) = node_path {
        if let Some(dir) = node.parent() {
            candidates.push(dir.join("node_modules/rea-agents/scripts/rea.mjs"));
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("lib/node_modules/rea-agents/scripts/rea.mjs"));
            }
        }
    }
    candidates.push(home.join("AppData/Roaming/npm/node_modules/rea-agents/scripts/rea.mjs"));
    candidates.push(PathBuf::from("/usr/local/lib/node_modules/rea-agents/scripts/rea.mjs"));
    candidates.push(PathBuf::from("/opt/homebrew/lib/node_modules/rea-agents/scripts/rea.mjs"));
    candidates.into_iter().find(|p| p.is_file()).map(|p| canonicalize_clean(&p))
}

fn build_global_mcp_json(repo_root: &Path, home: &Path) -> (Value, Vec<String>) {
    let abs_repo = canonicalize_clean(repo_root);
    let mut servers = Map::new();
    let mut registered = Vec::new();

    // 1. Native Rust MCP server (`bob-tuner mcp`) — zero runtime dependencies
    let exe = std::env::current_exe()
        .map(|p| canonicalize_clean(&p))
        .unwrap_or_else(|_| {
            let bin = if cfg!(windows) {
                "bob-tuner.exe"
            } else {
                "bob-tuner"
            };
            abs_repo.join("target").join("release").join(bin)
        });
    servers.insert(
        "pe-agents-ibm".to_string(),
        json!({
            "command": exe.to_string_lossy(),
            "args": ["mcp", "--repo", abs_repo.to_string_lossy()],
            "timeout": 60000,
            "alwaysAllow": [
                "list_agents",
                "get_agent",
                "list_skills",
                "get_skill",
                "get_tuning_policy",
                "search_knowledge"
            ]
        }),
    );
    registered.push("pe-agents-ibm".to_string());

    // 2. Optional auto-detected host MCPs (only registered when binaries exist locally)
    let node_bin = if cfg!(windows) {
        find_in_path(&["node.exe"])
    } else {
        find_in_path(&["node"])
    };
    let npx_bin = if cfg!(windows) {
        find_in_path(&["npx.cmd", "npx.exe"])
    } else {
        find_in_path(&["npx"])
    };

    if let (Some(node), Some(rea)) = (&node_bin, find_rea_script(home, node_bin.as_deref())) {
        servers.insert(
            "rea".to_string(),
            json!({
                "command": node.to_string_lossy(),
                "args": [rea.to_string_lossy(), "mcp"],
                "timeout": 120000,
                "alwaysAllow": [
                    "analyze_javascript_application",
                    "inspect_analysis_view",
                    "inspect_electron_fuses",
                    "inspect_artifact",
                    "inspect_pe_resources",
                    "inspect_pe_imports",
                    "inspect_pe_certificates",
                    "inspect_native_exports",
                    "inspect_go_binary",
                    "inspect_rust_binary",
                    "inspect_wasm_artifact",
                    "inspect_binary_layout",
                    "inspect_sqlite_database",
                    "open_binary",
                    "close_binary",
                    "binary_session",
                    "binary_overview",
                    "list_procedures",
                    "search_procedures",
                    "list_strings",
                    "search_strings",
                    "analyze_function",
                    "procedure_pseudo_code",
                    "trace_application_feature",
                    "get_evidence_bundle",
                    "list_unknowns"
                ]
            }),
        );
        registered.push("rea".to_string());
    }

    if let Some(npx) = &npx_bin {
        servers.insert(
            "chrome-devtools".to_string(),
            json!({
                "command": npx.to_string_lossy(),
                "args": ["-y", "chrome-devtools-mcp@latest"],
                "timeout": 60000,
                "alwaysAllow": [
                    "list_pages",
                    "take_snapshot",
                    "take_screenshot",
                    "list_console_messages",
                    "get_console_message",
                    "list_network_requests",
                    "get_network_request",
                    "get_css_styles",
                    "lighthouse_audit"
                ]
            }),
        );
        registered.push("chrome-devtools".to_string());

        servers.insert(
            "fetch".to_string(),
            json!({
                "command": npx.to_string_lossy(),
                "args": ["-y", "@modelcontextprotocol/server-fetch"],
                "timeout": 60000,
                "alwaysAllow": ["fetch"]
            }),
        );
        registered.push("fetch".to_string());
    }

    let go_bin = if cfg!(windows) {
        find_in_path(&["go.exe"])
    } else {
        find_in_path(&["go"])
    };
    if let Some(go_exe) = go_bin {
        servers.insert(
            "gopls".to_string(),
            json!({
                "command": go_exe.to_string_lossy(),
                "args": ["run", "golang.org/x/tools/gopls@latest", "mcp"],
                "timeout": 60000,
                "alwaysAllow": [
                    "go_diagnostics",
                    "go_file_context",
                    "go_package_api",
                    "go_search",
                    "go_symbol_references",
                    "go_workspace"
                ]
            }),
        );
        registered.push("gopls".to_string());
    }

    (json!({ "mcpServers": servers }), registered)
}

fn sync_skills_dir(repo_root: &Path, home: &Path, dry_run: bool) -> Result<usize, String> {
    let src_skills = repo_root.join("skills");
    if !src_skills.is_dir() {
        return Ok(0);
    }
    let dst_skills = home.join(".bob").join("skills");
    let skill_re = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$")
        .map_err(|e| format!("Invalid skill regex: {}", e))?;

    let mut count = 0usize;
    for entry in fs::read_dir(&src_skills)
        .map_err(|e| format!("Failed listing '{}': {}", src_skills.display(), e))?
        .filter_map(|e| e.ok())
    {
        let dir_path = entry.path();
        if !dir_path.is_dir() || !dir_path.join("SKILL.md").is_file() {
            continue;
        }
        let Some(name) = dir_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !skill_re.is_match(name) || name.len() > 64 {
            continue;
        }
        count += 1;
        if dry_run {
            continue;
        }
        let target_dir = dst_skills.join(name);
        for item in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            let p = item.path();
            let rel = p
                .strip_prefix(&dir_path)
                .map_err(|e| format!("strip_prefix error: {}", e))?;
            let out_path = target_dir.join(rel);
            if p.is_dir() {
                fs::create_dir_all(&out_path)
                    .map_err(|e| format!("mkdir '{}': {}", out_path.display(), e))?;
            } else if p.is_file() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("mkdir '{}': {}", parent.display(), e))?;
                }
                fs::copy(p, &out_path).map_err(|e| {
                    format!("copy '{}' -> '{}': {}", p.display(), out_path.display(), e)
                })?;
            }
        }
    }
    Ok(count)
}

fn sync_workspace_agents(
    workspace: &Path,
    agents: &[BobAgent],
    dry_run: bool,
) -> Result<(usize, usize), String> {
    let target_dir = workspace.join(".bob").join("agents");
    let mut legacy_removed = 0usize;

    if !dry_run {
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed creating '{}': {}", target_dir.display(), e))?;

        let legacy: Vec<PathBuf> = fs::read_dir(&target_dir)
            .map_err(|e| format!("Failed listing '{}': {}", target_dir.display(), e))?
            .filter_map(|e| e.ok().map(|d| d.path()))
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".agent.md"))
                    .unwrap_or(false)
            })
            .collect();

        if !legacy.is_empty() {
            let bak = workspace.join(".bob").join("agents-legacy-backup");
            fs::create_dir_all(&bak)
                .map_err(|e| format!("Failed creating legacy backup '{}': {}", bak.display(), e))?;
            for lf in legacy {
                if let Some(fname) = lf.file_name() {
                    let _ = fs::copy(&lf, bak.join(fname));
                }
                fs::remove_file(&lf)
                    .map_err(|e| format!("Failed removing legacy '{}': {}", lf.display(), e))?;
                legacy_removed += 1;
            }
        }

        for agent in agents {
            let compiled = render_bob_agent_md(agent);
            write_text_atomic(&target_dir.join(format!("{}.md", agent.file_stem)), &compiled)?;
        }
    }

    Ok((agents.len(), legacy_removed))
}

#[allow(clippy::too_many_arguments)]
fn sync_bob(
    repo_root: &Path,
    home: &Path,
    policy: &TurboPolicy,
    workspaces: &[PathBuf],
    all_modes: bool,
    model_override: Option<&str>,
    turbo_enabled: bool,
    dry_run: bool,
) -> Result<(), String> {
    let agents = load_repo_agents(repo_root)?;
    if agents.is_empty() {
        return Err("No agents found in repo/agents/".to_string());
    }

    let settings_dir = home.join(".bob").join("settings");
    let settings_path = settings_dir.join("settings.json");
    let custom_modes_path = settings_dir.join("custom_modes.yaml");
    let mcp_path = settings_dir.join("mcp.json");
    let global_rule_src = repo_root.join("rules").join("00-pe-agents-global.md");
    let global_rule_dst = home.join(".bob").join("rules").join("00-pe-agents-global.md");

    let backup_dir = if !dry_run {
        Some(backup_bob_settings(home)?)
    } else {
        None
    };

    let existing = read_json_object_or_empty(&settings_path)?;
    let (new_settings, active_model) =
        build_settings_json(&existing, policy, model_override, turbo_enabled);
    if !dry_run {
        write_json_atomic(&settings_path, &new_settings)?;
    }

    let (modes_yaml, modes_count) = build_custom_modes_yaml(&agents, policy, all_modes)?;
    if !dry_run {
        write_text_atomic(&custom_modes_path, &modes_yaml)?;
    }

    if global_rule_src.is_file() && !dry_run {
        let rule_txt = fs::read_to_string(&global_rule_src)
            .map_err(|e| format!("Failed reading global rule: {}", e))?;
        write_text_atomic(&global_rule_dst, &rule_txt)?;
    }

    let skills_count = sync_skills_dir(repo_root, home, dry_run)?;

    let (mcp_val, mcp_names) = build_global_mcp_json(repo_root, home);
    let mcp_text = serde_json::to_string_pretty(&mcp_val)
        .map_err(|e| format!("Failed serializing mcp.json: {}", e))?;
    if mcp_text.contains("${") {
        return Err("mcp.json must not contain ${VAR} templates (not expanded by IBM Bob)".to_string());
    }
    if !dry_run {
        write_text_atomic(&mcp_path, &format!("{}\n", mcp_text))?;
    }

    let mut ws_results = Vec::new();
    for ws in workspaces {
        if ws.is_dir() {
            let abs = canonicalize_clean(ws);
            let (cnt, leg) = sync_workspace_agents(&abs, &agents, dry_run)?;
            ws_results.push((abs, cnt, leg));
        }
    }

    println!("================================================================================");
    println!(
        "IBM BOB v2.1.0 — PE-AGENTS-IBM SYNC [{}]",
        if dry_run { "DRY-RUN" } else { "APPLIED" }
    );
    println!("================================================================================");
    if let Some(bak) = backup_dir {
        println!("  Backup SHA-256:    {}", bak.display());
    }
    println!(
        "  Turbo Mode:        {} ({} allow / {} deny, isCommandSecurityEnabled={})",
        if turbo_enabled { "ENABLED" } else { "DISABLED" },
        policy.approved_commands.len(),
        policy.denied_commands.len(),
        policy.is_command_security_enabled
    );
    println!(
        "  session.model:     {}",
        if active_model.is_empty() {
            "\"\" (default alias: premium-ide; all subagents pinned to model: premium)"
        } else {
            active_model.as_str()
        }
    );
    println!("  Modes in Picker:   {} -> {}", modes_count, custom_modes_path.display());
    println!("  Global Rules:      {}", global_rule_dst.display());
    println!("  Global Skills:     {} synced to ~/.bob/skills/", skills_count);
    println!("  Global MCPs ({}):   [{}] -> {}", mcp_names.len(), mcp_names.join(", "), mcp_path.display());
    for (ws, cnt, leg) in ws_results {
        println!(
            "  Workspace Agents:  {} compiled in {}/.bob/agents (legacy archived: {})",
            cnt,
            ws.display(),
            leg
        );
    }
    println!("================================================================================");
    Ok(())
}

fn print_status(home: &Path, workspaces: &[PathBuf]) -> Result<(), String> {
    let settings_dir = home.join(".bob").join("settings");
    let settings_path = settings_dir.join("settings.json");
    let modes_path = settings_dir.join("custom_modes.yaml");
    let mcp_path = settings_dir.join("mcp.json");
    let rules_dir = home.join(".bob").join("rules");
    let skills_dir = home.join(".bob").join("skills");

    println!("================================================================================");
    println!("IBM BOB v2.1.0 — CONFIGURATION AUDIT");
    println!("================================================================================");

    if settings_path.is_file() {
        let obj = read_json_object_or_empty(&settings_path)?;
        let default_mode = obj
            .get("session")
            .and_then(|s| s.get("defaultMode"))
            .and_then(|v| v.as_str())
            .unwrap_or("(default: agent)");
        let model = obj
            .get("session")
            .and_then(|s| s.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let auto_approve = obj
            .get("approval")
            .and_then(|a| a.get("autoApprovalEnabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let allow_cnt = obj
            .get("approval")
            .and_then(|a| a.get("allowedExecutors"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("approvedCommands"))
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let deny_cnt = obj
            .get("approval")
            .and_then(|a| a.get("allowedExecutors"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("deniedCommands"))
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        println!("  settings.json:     {}", settings_path.display());
        println!("    defaultMode:     {}", default_mode);
        println!(
            "    session.model:   {}",
            if model.is_empty() { "\"\" (premium-ide)" } else { model }
        );
        println!(
            "    autoApproval:    {} (allow={}, deny={})",
            if auto_approve { "ON" } else { "OFF" },
            allow_cnt,
            deny_cnt
        );
    } else {
        println!("  settings.json:     MISSING");
    }

    if modes_path.is_file() {
        let raw = fs::read_to_string(&modes_path).unwrap_or_default();
        let cnt = serde_yaml::from_str::<BobCustomModesFile>(&raw)
            .map(|f| f.custom_modes.len())
            .unwrap_or(0);
        println!("  custom_modes.yaml: {} mode(s)", cnt);
    } else {
        println!("  custom_modes.yaml: MISSING");
    }

    if mcp_path.is_file() {
        let obj = read_json_object_or_empty(&mcp_path)?;
        let names: Vec<String> = obj
            .get("mcpServers")
            .and_then(|m| m.as_object())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        println!("  mcp.json:          {} server(s): [{}]", names.len(), names.join(", "));
    } else {
        println!("  mcp.json:          MISSING");
    }

    let rules_cnt = fs::read_dir(&rules_dir)
        .map(|it| it.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count())
        .unwrap_or(0);
    let skills_cnt = fs::read_dir(&skills_dir)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| e.path().join("SKILL.md").is_file())
                .count()
        })
        .unwrap_or(0);
    println!("  ~/.bob/rules:      {} file(s)", rules_cnt);
    println!("  ~/.bob/skills:     {} skill(s)", skills_cnt);

    for ws in workspaces {
        let adir = ws.join(".bob").join("agents");
        let cnt = fs::read_dir(&adir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.ends_with(".md") && !n.ends_with(".agent.md"))
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);
        println!("  WS .bob/agents:    {} agent(s) in {}", cnt, ws.display());
    }
    println!("================================================================================");
    Ok(())
}

fn validate_repo(repo_root: &Path, policy: &TurboPolicy) -> Result<(), String> {
    let agents = load_repo_agents(repo_root)?;
    if agents.is_empty() {
        return Err("No agents found".to_string());
    }
    for a in &agents {
        if a.model != "premium" && a.model != "ultra" && a.model != "fast" {
            return Err(format!("Agent '{}' has invalid model tier '{}'", a.file_stem, a.model));
        }
        let compiled = render_bob_agent_md(a);
        if !compiled.contains("## Output Constraints") {
            return Err(format!("Agent '{}' missing ## Output Constraints", a.file_stem));
        }
    }

    let (yaml, curated_cnt) = build_custom_modes_yaml(&agents, policy, false)?;
    if !yaml.is_ascii() {
        return Err("Generated custom_modes.yaml contains non-ASCII characters".to_string());
    }
    let _roundtrip: BobCustomModesFile = serde_yaml::from_str(&yaml)
        .map_err(|e| format!("custom_modes.yaml roundtrip failed: {}", e))?;

    let skill_re = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$")
        .map_err(|e| format!("Regex error: {}", e))?;
    let mut skills_ok = 0usize;
    let skills_dir = repo_root.join("skills");
    if skills_dir.is_dir() {
        for entry in fs::read_dir(&skills_dir)
            .map_err(|e| format!("read_dir skills: {}", e))?
            .filter_map(|e| e.ok())
        {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !skill_re.is_match(&name) || name.len() > 64 {
                    return Err(format!("Skill directory '{}' violates Bob isValidSkillName", name));
                }
                if !entry.path().join("SKILL.md").is_file() {
                    return Err(format!("Skill '{}' missing SKILL.md", name));
                }
                skills_ok += 1;
            }
        }
    }

    println!(
        "[PASS] bob-tuner validate: {} agents (Bob TFi OK), {} curated modes (ASCII YAML OK), {} skills (Bob regex OK), {} allow / {} deny commands",
        agents.len(),
        curated_cnt,
        skills_ok,
        policy.approved_commands.len(),
        policy.denied_commands.len()
    );
    Ok(())
}

// ============================================================================
// Standalone Native Rust MCP Stdio Server (`bob-tuner mcp`) — JSON-RPC 2.0
// ============================================================================

fn is_safe_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

fn mcp_tools_catalog() -> Value {
    json!({
        "tools": [
            {
                "name": "list_agents",
                "description": "Lists all PE-AGENTS-IBM specialist agents available in ibm-bob-tuning.",
                "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
            },
            {
                "name": "get_agent",
                "description": "Returns the complete specification of a PE-AGENTS-IBM specialist agent.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Agent slug (e.g. pe-agents-ibm-rust-backend)" }
                    },
                    "required": ["name"],
                    "additionalProperties": false
                }
            },
            {
                "name": "list_skills",
                "description": "Lists all reusable PE-AGENTS-IBM skills available in ibm-bob-tuning.",
                "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
            },
            {
                "name": "get_skill",
                "description": "Returns the full SKILL.md content of a PE-AGENTS-IBM skill.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Skill slug (e.g. revisao-owasp, verification-harness)" }
                    },
                    "required": ["name"],
                    "additionalProperties": false
                }
            },
            {
                "name": "get_tuning_policy",
                "description": "Returns the active IBM Bob Turbo Mode allow/deny command policy and curated modes.",
                "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
            },
            {
                "name": "search_knowledge",
                "description": "Searches across all local PE-AGENTS-IBM agents, skills, and rules for a keyword or pattern.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Keyword or phrase to search" }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn execute_mcp_tool(repo_root: &Path, tool_name: &str, args: &Value) -> Result<String, String> {
    let agents_dir = repo_root.join("agents");
    let skills_dir = repo_root.join("skills");

    match tool_name {
        "list_agents" => {
            let mut names: Vec<String> = fs::read_dir(&agents_dir)
                .map_err(|e| format!("Failed listing agents: {}", e))?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|n| n.ends_with(".md"))
                .map(|n| n.trim_end_matches(".md").to_string())
                .collect();
            names.sort();
            Ok(format!(
                "PE-AGENTS-IBM Specialists ({}):\n{}",
                names.len(),
                names
                    .iter()
                    .map(|n| format!("- {}", n))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        "get_agent" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim())
                .ok_or_else(|| "Missing required argument 'name'".to_string())?;
            if !is_safe_identifier(name) {
                return Err("Invalid agent identifier".to_string());
            }
            let stem = name
                .strip_suffix(".agent.md")
                .or_else(|| name.strip_suffix(".md"))
                .unwrap_or(name);
            let target = agents_dir.join(format!("{}.md", stem));
            if !target.is_file() {
                return Err(format!("Agent '{}' not found", name));
            }
            fs::read_to_string(&target)
                .map_err(|e| format!("Failed reading agent '{}': {}", name, e))
        }
        "list_skills" => {
            let mut names: Vec<String> = fs::read_dir(&skills_dir)
                .map_err(|e| format!("Failed listing skills: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("SKILL.md").is_file())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            names.sort();
            Ok(format!(
                "PE-AGENTS-IBM Skills ({}):\n{}",
                names.len(),
                names
                    .iter()
                    .map(|n| format!("- {}", n))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        "get_skill" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim())
                .ok_or_else(|| "Missing required argument 'name'".to_string())?;
            if !is_safe_identifier(name) {
                return Err("Invalid skill identifier".to_string());
            }
            let target = skills_dir.join(name).join("SKILL.md");
            if !target.is_file() {
                return Err(format!("Skill '{}' not found", name));
            }
            fs::read_to_string(&target)
                .map_err(|e| format!("Failed reading skill '{}': {}", name, e))
        }
        "get_tuning_policy" => {
            let p = repo_root.join("config").join("turbo-policy.json");
            if p.is_file() {
                fs::read_to_string(&p).map_err(|e| format!("Failed reading policy: {}", e))
            } else {
                Ok(EMBEDDED_POLICY_JSON.to_string())
            }
        }
        "search_knowledge" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "Missing required argument 'query'".to_string())?;
            let mut hits = Vec::new();
            for sub in ["agents", "skills", "rules"] {
                let dir = repo_root.join(sub);
                if !dir.is_dir() {
                    continue;
                }
                for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if !p.is_file() || p.extension().and_then(|e| e.to_str()) != Some("md") {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(p) {
                        for (idx, line) in content.lines().enumerate() {
                            if line.to_ascii_lowercase().contains(&query) {
                                let rel = p.strip_prefix(repo_root).unwrap_or(p);
                                hits.push(format!("{}:{}: {}", rel.display(), idx + 1, line.trim()));
                                if hits.len() >= 25 {
                                    return Ok(hits.join("\n"));
                                }
                                break;
                            }
                        }
                    }
                }
            }
            if hits.is_empty() {
                Ok(format!("No matches found for '{}'.", query))
            } else {
                Ok(hits.join("\n"))
            }
        }
        other => Err(format!("Unknown MCP tool: '{}'", other)),
    }
}

fn handle_mcp_request(repo_root: &Path, req: &Value) -> Option<Value> {
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let id = req.get("id").cloned();

    if id.is_none() || method == "notifications/initialized" || method == "notifications/cancelled" {
        return None;
    }
    let id_val = id.unwrap_or(Value::Null);

    match method {
        "initialize" => {
            let proto = req
                .get("params")
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");
            Some(json!({
                "jsonrpc": "2.0",
                "id": id_val,
                "result": {
                    "protocolVersion": proto,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": {
                        "name": "pe-agents-ibm",
                        "version": "1.0.0-rust"
                    },
                    "instructions": "Standalone PE-AGENTS-IBM specialist catalog, skills, and IBM Bob tuning policy."
                }
            }))
        }
        "ping" => Some(json!({ "jsonrpc": "2.0", "id": id_val, "result": {} })),
        "tools/list" => Some(json!({ "jsonrpc": "2.0", "id": id_val, "result": mcp_tools_catalog() })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match execute_mcp_tool(repo_root, tool_name, &tool_args) {
                Ok(text) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "result": {
                        "content": [{ "type": "text", "text": text }],
                        "isError": false
                    }
                })),
                Err(msg) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Error: {}", msg) }],
                        "isError": true
                    }
                })),
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "error": { "code": -32601, "message": format!("Method not found: {}", method) }
        })),
    }
}

fn run_mcp_stdio_server(repo_root: &Path) -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for line_res in stdin.lock().lines() {
        let line = line_res.map_err(|e| format!("MCP stdin read error: {}", e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                let _ = writeln!(out, "{}", err_resp);
                let _ = out.flush();
                continue;
            }
        };
        if let Some(resp) = handle_mcp_request(repo_root, &req) {
            writeln!(out, "{}", resp).map_err(|e| format!("MCP stdout write error: {}", e))?;
            out.flush().map_err(|e| format!("MCP stdout flush error: {}", e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_bob_command_matcher(cmd: &str, approved: &[String], denied: &[String]) -> &'static str {
        let longest = |patterns: &[String]| -> Option<usize> {
            let cmd_words: Vec<&str> = cmd.split_whitespace().collect();
            let mut best: Option<usize> = None;
            for pat in patterns {
                let pat_words: Vec<&str> = pat.split_whitespace().collect();
                if pat_words.is_empty() || pat_words.len() > cmd_words.len() {
                    continue;
                }
                if pat_words.iter().zip(cmd_words.iter()).all(|(a, b)| a == b)
                    && best.map(|b| pat_words.len() > b).unwrap_or(true)
                {
                    best = Some(pat_words.len());
                }
            }
            best
        };
        match (longest(approved), longest(denied)) {
            (None, None) => "ask",
            (Some(_), None) => "allow",
            (None, Some(_)) => "deny",
            (Some(a), Some(d)) => {
                if a >= d {
                    "allow"
                } else {
                    "deny"
                }
            }
        }
    }

    #[test]
    fn test_embedded_policy_command_matcher_parity() {
        let policy: TurboPolicy =
            serde_json::from_str(EMBEDDED_POLICY_JSON).expect("valid embedded policy");
        assert!(policy.is_command_security_enabled);

        assert_eq!(
            eval_bob_command_matcher("node -v", &policy.approved_commands, &policy.denied_commands),
            "allow"
        );
        assert_eq!(
            eval_bob_command_matcher("npm -v", &policy.approved_commands, &policy.denied_commands),
            "allow"
        );
        assert_eq!(
            eval_bob_command_matcher("npm create vite@latest app", &policy.approved_commands, &policy.denied_commands),
            "allow"
        );
        assert_eq!(
            eval_bob_command_matcher("cargo test", &policy.approved_commands, &policy.denied_commands),
            "allow"
        );
        assert_eq!(
            eval_bob_command_matcher("git status", &policy.approved_commands, &policy.denied_commands),
            "allow"
        );

        assert_eq!(
            eval_bob_command_matcher("rm -rf /tmp/x", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("git push --force origin main", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("git reset --hard HEAD~1", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("npm publish", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("terraform destroy", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("kubectl delete namespace prod", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
        assert_eq!(
            eval_bob_command_matcher("DROP TABLE users", &policy.approved_commands, &policy.denied_commands),
            "deny"
        );
    }

    #[test]
    fn test_mcp_protocol_and_path_traversal_guard() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let init = handle_mcp_request(
            &repo,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05"}}),
        )
        .expect("init reply");
        assert_eq!(init["result"]["serverInfo"]["name"], "pe-agents-ibm");

        let traversal = handle_mcp_request(
            &repo,
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "get_agent", "arguments": {"name": "../../Cargo.toml"}}}),
        )
        .expect("traversal reply");
        assert_eq!(traversal["result"]["isError"], true);
    }

    #[test]
    fn test_sanitize_ascii_clean() {
        let out = sanitize_ascii("“Bob” — validação… → produção");
        assert!(out.is_ascii());
        assert_eq!(out, "\"Bob\" - validacao... -> producao");
    }
}
