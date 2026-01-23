// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use base64::Engine;
use toml::map::Map as TomlMap;
use toml::Value as TomlValue;
use url::Url;

#[derive(Clone)]
struct SourceConfig {
    id: &'static str,
    label: &'static str,
    install_root: PathBuf,
    root: PathBuf,
    core_files: Vec<&'static str>,
}

#[derive(Clone, Copy)]
enum McpKind {
    CodexToml,
    ClaudeJson,
    OpenCodeJson,
}

#[derive(Clone)]
struct McpSourceConfig {
    id: &'static str,
    label: &'static str,
    format: &'static str,
    kind: McpKind,
    install_root: PathBuf,
    primary_path: PathBuf,
    read_paths: Vec<PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillSource {
    id: String,
    label: String,
    root: String,
    exists: bool,
    skills: Vec<SkillItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillItem {
    id: String,
    name: String,
    description: String,
    path: String,
    core_file: String,
    core_file_path: String,
    source_url: Option<String>,
    source_id: String,
    metadata: HashMap<String, String>,
    body: String,
    last_modified: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillTreeNode {
    name: String,
    path: String,
    kind: String,
    children: Vec<SkillTreeNode>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct McpSource {
    id: String,
    label: String,
    path: String,
    format: String,
    exists: bool,
    servers: Vec<McpServer>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct McpServer {
    id: String,
    config: JsonValue,
}

const SKILL_SOURCE_FILENAME: &str = ".skill-source.json";

#[derive(Clone)]
struct GithubLocation {
    owner: String,
    repo: String,
    branch: Option<String>,
    path: String,
}

#[derive(Deserialize)]
struct GithubContentEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    item_type: String,
    sha: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallSkillInput {
    source_id: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteSkillInput {
    source_id: String,
    skill_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkillTreeInput {
    source_id: String,
    skill_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncSkillInput {
    source_id: String,
    skill_id: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertMcpJsonInput {
    source_id: String,
    json: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteMcpInput {
    source_id: String,
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncAgentsInput {
    source_id: String,
    target_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncResult {
    added: usize,
    skipped: usize,
}

fn resolve_home() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    dirs::home_dir().ok_or_else(|| "Unable to resolve home directory".to_string())
}

fn source_configs(home: &Path) -> Vec<SourceConfig> {
    let skill_md = vec!["SKILL.md"];
    let antigravity_files = vec!["manifest.json", "SKILL.md"];
    let codebuddy_files = vec![".cb-rules", "SKILL.md"];
    let kiro_files = vec!["instructions.md"];
    let qoder_files = vec!["config.yaml"];
    let antigravity_root = {
        let gemini_root = home.join(".gemini").join("antigravity");
        let legacy_root = home.join(".antigravity");
        if gemini_root.exists() || !legacy_root.exists() {
            gemini_root
        } else {
            legacy_root
        }
    };

    vec![
        SourceConfig {
            id: "claude-user",
            label: "Claude Code",
            install_root: home.join(".claude"),
            root: home.join(".claude").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "roo-user",
            label: "Roo Code (Cline)",
            install_root: home.join(".roo"),
            root: home.join(".roo").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "copilot-user",
            label: "GitHub Copilot",
            install_root: home.join(".copilot"),
            root: home.join(".copilot").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "cursor-user",
            label: "Cursor",
            install_root: home.join(".cursor"),
            root: home.join(".cursor").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "opencode-user",
            label: "OpenCode",
            install_root: home.join(".config").join("opencode"),
            root: home.join(".config").join("opencode").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "gemini-user",
            label: "Gemini CLI",
            install_root: home.join(".gemini"),
            root: home.join(".gemini").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "codex-user",
            label: "Codex",
            install_root: home.join(".codex"),
            root: home.join(".codex").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "trae-user",
            label: "Trae",
            install_root: home.join(".trae"),
            root: home.join(".trae").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "goose-user",
            label: "Goose",
            install_root: home.join(".config").join("goose"),
            root: home.join(".config").join("goose").join("skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "standard-user",
            label: "Common Standard",
            install_root: home.join(".skills"),
            root: home.join(".skills"),
            core_files: skill_md.clone(),
        },
        SourceConfig {
            id: "antigravity-user",
            label: "Antigravity",
            install_root: antigravity_root.clone(),
            root: antigravity_root.join("skills"),
            core_files: antigravity_files.clone(),
        },
        SourceConfig {
            id: "kiro-user",
            label: "Kiro",
            install_root: home.join(".kiro"),
            root: home.join(".kiro").join("skills"),
            core_files: kiro_files.clone(),
        },
        SourceConfig {
            id: "qoder-user",
            label: "Qoder",
            install_root: home.join(".qoder"),
            root: home.join(".qoder").join("skills"),
            core_files: qoder_files.clone(),
        },
        SourceConfig {
            id: "codebuddy-user",
            label: "CodeBuddy",
            install_root: home.join(".codebuddy"),
            root: home.join(".codebuddy").join("skills"),
            core_files: codebuddy_files.clone(),
        },
    ]
}

fn mcp_source_configs(home: &Path) -> Vec<McpSourceConfig> {
    let claude_primary = home.join(".claude").join(".mcp.json");
    let claude_alt = home.join(".claude").join("mcp.json");
    let codex_path = home.join(".codex").join("config.toml");
    let opencode_path = home
        .join(".config")
        .join("opencode")
        .join("opencode.json");
    let roo_path = home.join(".roo").join("mcp.json");
    let copilot_path = home.join(".copilot").join("mcp.json");
    let cursor_path = home.join(".cursor").join("mcp.json");
    let gemini_path = home.join(".gemini").join("mcp.json");
    let trae_path = home.join(".trae").join("mcp.json");
    let goose_path = home.join(".config").join("goose").join("mcp.json");
    let antigravity_primary = home
        .join(".gemini")
        .join("antigravity")
        .join("mcp_config.json");
    let antigravity_legacy = home.join(".antigravity").join("mcp.json");
    let antigravity_path = if antigravity_primary.exists() || !antigravity_legacy.exists() {
        antigravity_primary.clone()
    } else {
        antigravity_legacy.clone()
    };
    let antigravity_root = antigravity_path
        .parent()
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(|| home.join(".gemini").join("antigravity"));
    let kiro_path = home.join(".kiro").join("mcp.json");
    let qoder_path = home.join(".qoder").join("mcp.json");
    let codebuddy_path = home.join(".codebuddy").join("mcp.json");

    vec![
        McpSourceConfig {
            id: "claude",
            label: "Claude Code",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".claude"),
            primary_path: claude_primary.clone(),
            read_paths: vec![claude_primary, claude_alt],
        },
        McpSourceConfig {
            id: "roo",
            label: "Roo Code (Cline)",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".roo"),
            primary_path: roo_path.clone(),
            read_paths: vec![roo_path],
        },
        McpSourceConfig {
            id: "copilot",
            label: "GitHub Copilot",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".copilot"),
            primary_path: copilot_path.clone(),
            read_paths: vec![copilot_path],
        },
        McpSourceConfig {
            id: "cursor",
            label: "Cursor",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".cursor"),
            primary_path: cursor_path.clone(),
            read_paths: vec![cursor_path],
        },
        McpSourceConfig {
            id: "gemini",
            label: "Gemini CLI",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".gemini"),
            primary_path: gemini_path.clone(),
            read_paths: vec![gemini_path],
        },
        McpSourceConfig {
            id: "codex",
            label: "Codex",
            format: "toml",
            kind: McpKind::CodexToml,
            install_root: home.join(".codex"),
            primary_path: codex_path.clone(),
            read_paths: vec![codex_path],
        },
        McpSourceConfig {
            id: "opencode",
            label: "OpenCode",
            format: "json",
            kind: McpKind::OpenCodeJson,
            install_root: home.join(".config").join("opencode"),
            primary_path: opencode_path.clone(),
            read_paths: vec![opencode_path],
        },
        McpSourceConfig {
            id: "trae",
            label: "Trae",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".trae"),
            primary_path: trae_path.clone(),
            read_paths: vec![trae_path],
        },
        McpSourceConfig {
            id: "goose",
            label: "Goose",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".config").join("goose"),
            primary_path: goose_path.clone(),
            read_paths: vec![goose_path],
        },
        McpSourceConfig {
            id: "antigravity",
            label: "Antigravity",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: antigravity_root,
            primary_path: antigravity_path.clone(),
            read_paths: vec![antigravity_primary, antigravity_legacy],
        },
        McpSourceConfig {
            id: "kiro",
            label: "Kiro",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".kiro"),
            primary_path: kiro_path.clone(),
            read_paths: vec![kiro_path],
        },
        McpSourceConfig {
            id: "qoder",
            label: "Qoder",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".qoder"),
            primary_path: qoder_path.clone(),
            read_paths: vec![qoder_path],
        },
        McpSourceConfig {
            id: "codebuddy",
            label: "CodeBuddy",
            format: "json",
            kind: McpKind::ClaudeJson,
            install_root: home.join(".codebuddy"),
            primary_path: codebuddy_path.clone(),
            read_paths: vec![codebuddy_path],
        },
    ]
}

fn resolve_read_path(config: &McpSourceConfig) -> PathBuf {
    for path in &config.read_paths {
        if path.exists() {
            return path.clone();
        }
    }
    config.primary_path.clone()
}

fn parse_frontmatter(raw: &str) -> (HashMap<String, String>, String) {
    let mut metadata = HashMap::new();
    let mut lines = raw.lines();

    if !matches!(lines.next(), Some(line) if line.trim() == "---") {
        return (metadata, raw.to_string());
    }

    let mut frontmatter = Vec::new();
    let mut found_close = false;

    for line in lines.by_ref() {
        if line.trim() == "---" {
            found_close = true;
            break;
        }
        frontmatter.push(line);
    }

    if !found_close {
        return (HashMap::new(), raw.to_string());
    }

    for line in frontmatter {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            metadata.insert(key.to_string(), value.trim().to_string());
        }
    }

    let body = lines.collect::<Vec<_>>().join("\n");
    (metadata, body.trim_start().to_string())
}

fn extract_description(body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        return trimmed.to_string();
    }
    String::new()
}

fn load_skill(
    skill_dir: &Path,
    core_file_path: &Path,
    core_file_name: &str,
    source: &SourceConfig,
) -> Result<SkillItem, String> {
    let raw = fs::read_to_string(core_file_path)
        .map_err(|err| format!("Failed to read {}: {}", core_file_path.display(), err))?;
    let is_markdown = core_file_name.ends_with(".md");
    let (metadata, body) = if is_markdown {
        parse_frontmatter(&raw)
    } else {
        (HashMap::new(), raw.clone())
    };
    let source_url = read_skill_source_url(skill_dir);
    let dir_name = skill_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill");

    let name = metadata
        .get("name")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| dir_name.to_string());
    let description = metadata
        .get("description")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| if is_markdown { extract_description(&body) } else { String::new() });

    let skill_md_path = skill_dir.join("SKILL.md");
    let last_modified = fs::metadata(skill_md_path)
        .or_else(|_| fs::metadata(core_file_path))
        .and_then(|data| data.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    Ok(SkillItem {
        id: dir_name.to_string(),
        name,
        description,
        path: skill_dir.display().to_string(),
        core_file: core_file_name.to_string(),
        core_file_path: core_file_path.display().to_string(),
        source_url,
        source_id: source.id.to_string(),
        metadata,
        body,
        last_modified,
    })
}

fn read_skills(source: &SourceConfig) -> Vec<SkillItem> {
    let mut skills = Vec::new();

    let entries = match fs::read_dir(&source.root) {
        Ok(entries) => entries,
        Err(_) => return skills,
    };

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if !file_type.is_dir() {
                continue;
            }
        }

        let path = entry.path();
        let core_file = find_core_file(&path, &source.core_files);
        if let Some((core_file_path, core_file_name)) = core_file {
            if let Ok(skill) = load_skill(&path, &core_file_path, &core_file_name, source) {
                skills.push(skill);
            }
        }
    }

    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    skills
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|err| format!("Failed to create {}: {}", dest.display(), err))?;
    let entries = fs::read_dir(src)
        .map_err(|err| format!("Failed to read {}: {}", src.display(), err))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("Failed to read entry: {}", err))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("Failed to read file type: {}", err))?;
        let source_path = entry.path();
        let target_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|err| {
                format!(
                    "Failed to copy {} to {}: {}",
                    source_path.display(),
                    target_path.display(),
                    err
                )
            })?;
        }
    }

    Ok(())
}

fn find_core_file(skill_dir: &Path, core_files: &[&str]) -> Option<(PathBuf, String)> {
    for file in core_files {
        let path = skill_dir.join(file);
        if path.is_file() {
            return Some((path, file.to_string()));
        }
    }
    None
}

fn build_skill_tree(path: &Path) -> Result<SkillTreeNode, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|err| format!("Failed to read metadata {}: {}", path.display(), err))?;
    let file_type = metadata.file_type();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let kind = if file_type.is_dir() {
        "dir"
    } else if file_type.is_symlink() {
        "link"
    } else {
        "file"
    };

    let mut children = Vec::new();
    if file_type.is_dir() {
        let entries = fs::read_dir(path)
            .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;
        let mut items: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
        items.sort_by(|a, b| {
            let a_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            if a_dir != b_dir {
                return b_dir.cmp(&a_dir);
            }
            a.file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase())
        });

        for entry in items {
            let path = entry.path();
            let child = build_skill_tree(&path)?;
            children.push(child);
        }
    }

    Ok(SkillTreeNode {
        name,
        path: path.display().to_string(),
        kind: kind.to_string(),
        children,
    })
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        return format!("skill-{}", stamp);
    }
    trimmed
}

fn parse_skill_urls(input: &str, core_file: &str) -> Result<Vec<String>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL is required".to_string());
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }

    let parsed = Url::parse(trimmed).map_err(|_| "Invalid URL".to_string())?;
    let host = parsed
        .host_str()
        .unwrap_or("")
        .trim_start_matches("www.");
    let segments: Vec<&str> = parsed
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    if host == "raw.githubusercontent.com" {
        let mut url = trimmed.trim_end_matches('/').to_string();
        if !url.ends_with(core_file) {
            url.push('/');
            url.push_str(core_file);
        }
        return Ok(vec![url]);
    }

    if host == "github.com" {
        if segments.len() < 2 {
            return Err("GitHub URL must include owner and repo".to_string());
        }
        let owner = segments[0];
        let repo = segments[1].trim_end_matches(".git");

        if segments.len() >= 4 && (segments[2] == "tree" || segments[2] == "blob") {
            let branch = segments[3];
            let subpath = segments[4..].join("/");
            let file_path = if subpath.is_empty() {
                core_file.to_string()
            } else if subpath.ends_with(core_file) {
                subpath
            } else {
                format!("{}/{}", subpath, core_file)
            };
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repo, branch, file_path
            );
            return Ok(vec![url]);
        }

        let subpath = if segments.len() > 2 {
            segments[2..].join("/")
        } else {
            String::new()
        };
        let file_path = if subpath.is_empty() {
            core_file.to_string()
        } else if subpath.ends_with(core_file) {
            subpath
        } else {
            format!("{}/{}", subpath, core_file)
        };
        return Ok(vec![
            format!(
                "https://raw.githubusercontent.com/{}/{}/main/{}",
                owner, repo, file_path
            ),
            format!(
                "https://raw.githubusercontent.com/{}/{}/master/{}",
                owner, repo, file_path
            ),
        ]);
    }

    let mut url = trimmed.trim_end_matches('/').to_string();
    if !url.ends_with(core_file) {
        url.push('/');
        url.push_str(core_file);
    }
    Ok(vec![url])
}

fn read_skill_source_url(skill_dir: &Path) -> Option<String> {
    let path = skill_dir.join(SKILL_SOURCE_FILENAME);
    let content = fs::read_to_string(path).ok()?;
    let value: JsonValue = serde_json::from_str(&content).ok()?;
    value
        .get("url")
        .and_then(|item| item.as_str())
        .map(|item| item.to_string())
}

fn write_skill_source_url(skill_dir: &Path, url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let path = skill_dir.join(SKILL_SOURCE_FILENAME);
    let mut map = JsonMap::new();
    map.insert("url".to_string(), JsonValue::String(trimmed.to_string()));
    let content = serde_json::to_string_pretty(&JsonValue::Object(map))
        .map_err(|err| format!("Failed to serialize JSON: {}", err))?;
    fs::write(&path, format!("{}\n", content))
        .map_err(|err| format!("Failed to write {}: {}", path.display(), err))?;
    Ok(())
}

fn parse_github_location(input: &str) -> Result<GithubLocation, String> {
    let trimmed = input.trim();
    let parsed = Url::parse(trimmed).map_err(|_| "Invalid URL".to_string())?;
    let host = parsed
        .host_str()
        .unwrap_or("")
        .trim_start_matches("www.");
    let segments: Vec<&str> = parsed
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if host == "raw.githubusercontent.com" {
        if segments.len() < 3 {
            return Err("Raw GitHub URL must include owner/repo/branch".to_string());
        }
        let owner = segments[0].to_string();
        let repo = segments[1].trim_end_matches(".git").to_string();
        let branch = Some(segments[2].to_string());
        let path = if segments.len() > 3 {
            segments[3..].join("/")
        } else {
            String::new()
        };
        return Ok(GithubLocation {
            owner,
            repo,
            branch,
            path,
        });
    }

    if host != "github.com" {
        return Err("Not a GitHub URL".to_string());
    }
    if segments.len() < 2 {
        return Err("GitHub URL must include owner and repo".to_string());
    }

    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    if segments.len() >= 4 && (segments[2] == "tree" || segments[2] == "blob") {
        let branch = Some(segments[3].to_string());
        let mut path = segments[4..].join("/");
        if segments[2] == "blob" {
            if let Some((dir, _)) = path.rsplit_once('/') {
                path = dir.to_string();
            } else {
                path.clear();
            }
        }
        return Ok(GithubLocation {
            owner,
            repo,
            branch,
            path,
        });
    }

    let path = if segments.len() > 2 {
        segments[2..].join("/")
    } else {
        String::new()
    };

    Ok(GithubLocation {
        owner,
        repo,
        branch: None,
        path,
    })
}

fn github_token() -> Option<String> {
    std::env::var("SKILL_GITHUB_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("GITHUB_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(|| {
            std::env::var("GH_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn github_request(agent: &ureq::Agent, url: &str) -> ureq::Request {
    let mut request = agent
        .get(url)
        .set("Accept", "application/vnd.github+json");
    if let Some(token) = github_token() {
        request = request.set("Authorization", &format!("Bearer {}", token));
    }
    request
}

fn read_json_response(response: ureq::Response) -> Result<JsonValue, String> {
    let body = response
        .into_string()
        .map_err(|err| format!("Failed to read response: {}", err))?;
    serde_json::from_str(&body).map_err(|err| format!("Invalid JSON: {}", err))
}

fn github_contents_url(owner: &str, repo: &str, path: &str, branch: &str) -> String {
    if path.is_empty() {
        format!(
            "https://api.github.com/repos/{}/{}/contents?ref={}",
            owner, repo, branch
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, path, branch
        )
    }
}

fn fetch_github_default_branch(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let response = github_request(agent, &url)
        .call()
        .map_err(|err| format!("Failed to read GitHub repo info: {}", err))?;
    let value = read_json_response(response)
        .map_err(|err| format!("Invalid GitHub response: {}", err))?;
    value
        .get("default_branch")
        .and_then(|item| item.as_str())
        .map(|item| item.to_string())
        .ok_or_else(|| "Missing default_branch in GitHub response".to_string())
}

fn github_branch_candidates(agent: &ureq::Agent, location: &GithubLocation) -> Vec<String> {
    if let Some(branch) = &location.branch {
        return vec![branch.clone()];
    }
    let mut branches = Vec::new();
    if let Ok(default_branch) =
        fetch_github_default_branch(agent, &location.owner, &location.repo)
    {
        branches.push(default_branch);
    }
    for candidate in ["main", "master"] {
        if !branches.iter().any(|branch| branch == candidate) {
            branches.push(candidate.to_string());
        }
    }
    if branches.is_empty() {
        branches.push("main".to_string());
    }
    branches
}

fn fetch_github_contents(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
) -> Result<Vec<GithubContentEntry>, String> {
    let url = github_contents_url(owner, repo, path, branch);
    let response = github_request(agent, &url)
        .call()
        .map_err(|err| format!("Failed to read GitHub contents: {}", err))?;
    let value = read_json_response(response)
        .map_err(|err| format!("Invalid GitHub response: {}", err))?;
    match value {
        JsonValue::Array(items) => items
            .into_iter()
            .map(|item| {
                serde_json::from_value(item)
                    .map_err(|err| format!("Invalid GitHub entry: {}", err))
            })
            .collect(),
        JsonValue::Object(_) => {
            let entry: GithubContentEntry = serde_json::from_value(value)
                .map_err(|err| format!("Invalid GitHub entry: {}", err))?;
            Ok(vec![entry])
        }
        _ => Err("Unexpected GitHub response".to_string()),
    }
}

fn decode_base64_payload(content: &str) -> Result<Vec<u8>, String> {
    let cleaned = content.replace('\n', "");
    base64::engine::general_purpose::STANDARD
        .decode(cleaned.as_bytes())
        .map_err(|err| format!("Failed to decode base64 payload: {}", err))
}

fn fetch_github_blob_content(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
    sha: &str,
) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/blobs/{}",
        owner, repo, sha
    );
    let response = github_request(agent, &url)
        .call()
        .map_err(|err| format!("Failed to read GitHub blob: {}", err))?;
    let value = read_json_response(response)
        .map_err(|err| format!("Invalid GitHub response: {}", err))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "Invalid GitHub blob response".to_string())?;
    let content = obj
        .get("content")
        .and_then(|item| item.as_str())
        .ok_or_else(|| "Missing content in GitHub blob response".to_string())?;
    let encoding = obj
        .get("encoding")
        .and_then(|item| item.as_str())
        .unwrap_or("base64");
    if encoding != "base64" {
        return Err("Unsupported GitHub blob encoding".to_string());
    }
    decode_base64_payload(content)
}

fn fetch_github_file_content(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
) -> Result<Vec<u8>, String> {
    let url = github_contents_url(owner, repo, path, branch);
    let response = github_request(agent, &url)
        .call()
        .map_err(|err| format!("Failed to read GitHub file: {}", err))?;
    let value = read_json_response(response)
        .map_err(|err| format!("Invalid GitHub response: {}", err))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "Invalid GitHub file response".to_string())?;
    if let Some(content) = obj.get("content").and_then(|item| item.as_str()) {
        let encoding = obj
            .get("encoding")
            .and_then(|item| item.as_str())
            .unwrap_or("base64");
        if encoding != "base64" {
            return Err("Unsupported GitHub file encoding".to_string());
        }
        return decode_base64_payload(content);
    }
    if let Some(sha) = obj.get("sha").and_then(|item| item.as_str()) {
        return fetch_github_blob_content(agent, owner, repo, sha);
    }
    Err("Missing content in GitHub file response".to_string())
}

fn write_bytes_to_path(bytes: &[u8], dest_path: &Path) -> Result<(), String> {
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {}", parent.display(), err))?;
    }
    fs::write(dest_path, bytes)
        .map_err(|err| format!("Failed to write {}: {}", dest_path.display(), err))?;
    Ok(())
}

fn download_github_directory(
    agent: &ureq::Agent,
    location: &GithubLocation,
    branch: &str,
    dest_dir: &Path,
) -> Result<(), String> {
    download_github_directory_recursive(
        agent,
        &location.owner,
        &location.repo,
        branch,
        &location.path,
        dest_dir,
    )
}

fn download_github_directory_recursive(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
    branch: &str,
    repo_path: &str,
    dest_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(dest_dir)
        .map_err(|err| format!("Failed to create {}: {}", dest_dir.display(), err))?;
    let entries = fetch_github_contents(agent, owner, repo, repo_path, branch)?;
    for entry in entries {
        match entry.item_type.as_str() {
            "dir" => {
                let next_dest = dest_dir.join(&entry.name);
                download_github_directory_recursive(
                    agent,
                    owner,
                    repo,
                    branch,
                    &entry.path,
                    &next_dest,
                )?;
            }
            "file" => {
                let dest_path = dest_dir.join(&entry.name);
                let bytes = if let Some(sha) = entry.sha.as_deref() {
                    fetch_github_blob_content(agent, owner, repo, sha)?
                } else {
                    fetch_github_file_content(agent, owner, repo, &entry.path, branch)?
                };
                write_bytes_to_path(&bytes, &dest_path)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn github_file_path(location: &GithubLocation, core_file: &str) -> String {
    let mut file_path = location.path.clone();
    if file_path.is_empty() {
        file_path = core_file.to_string();
    } else if !file_path.ends_with(core_file) {
        file_path = format!("{}/{}", file_path, core_file);
    }
    file_path
}

fn fallback_name_from_url(input: &str, core_file: &str) -> String {
    if let Ok(parsed) = Url::parse(input) {
        let mut segments: Vec<&str> = parsed
            .path()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        if let Some(last) = segments.pop() {
            if last.eq_ignore_ascii_case(core_file) {
                if let Some(prev) = segments.pop() {
                    return prev.to_string();
                }
            }
            return last.to_string();
        }
    }
    "skill".to_string()
}

fn fetch_skill_content(urls: Vec<String>) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .user_agent("Ananke/0.1")
        .build();
    let mut last_error = None;

    for url in urls {
        match agent.get(&url).call() {
            Ok(response) => {
                if response.status() == 200 {
                    let body = response
                        .into_string()
                        .map_err(|err| format!("Failed to read response: {}", err))?;
                    if body.trim().is_empty() {
                        return Err("SKILL.md is empty".to_string());
                    }
                    return Ok(body);
                }
                last_error = Some(format!("Unexpected status {}", response.status()));
            }
            Err(err) => {
                last_error = Some(format!("Request failed: {}", err));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Unable to download SKILL.md".to_string()))
}

fn line_col_from_index(input: &str, index: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;

    for (offset, ch) in input.char_indices() {
        if offset >= index {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else if ch != '\r' {
            col += 1;
        }
    }

    (line, col)
}

fn load_toml_value(path: &Path) -> Result<TomlValue, String> {
    if !path.exists() {
        return Ok(TomlValue::Table(TomlMap::new()));
    }
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;
    if content.trim().is_empty() {
        return Ok(TomlValue::Table(TomlMap::new()));
    }
    content
        .parse::<TomlValue>()
        .map_err(|err| {
            let location = err
                .span()
                .map(|span| {
                    let (line, col) = line_col_from_index(&content, span.start);
                    format!("line {}, column {}", line, col)
                })
                .unwrap_or_else(|| "unknown location".to_string());
            format!(
                "Invalid TOML in {} ({}): {}",
                path.display(),
                location,
                err
            )
        })
}

fn save_toml_value(path: &Path, value: &TomlValue) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {}", parent.display(), err))?;
    }
    let content = toml::to_string_pretty(value)
        .map_err(|err| format!("Failed to serialize TOML: {}", err))?;
    fs::write(path, content)
        .map_err(|err| format!("Failed to write {}: {}", path.display(), err))
}

fn load_json_value(path: &Path) -> Result<JsonValue, String> {
    if !path.exists() {
        return Ok(JsonValue::Object(JsonMap::new()));
    }
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;
    if content.trim().is_empty() {
        return Ok(JsonValue::Object(JsonMap::new()));
    }
    serde_json::from_str(&content).map_err(|err| {
        format!(
            "Invalid JSON in {} (line {}, column {}): {}",
            path.display(),
            err.line(),
            err.column(),
            err
        )
    })
}

fn save_json_value(path: &Path, value: &JsonValue) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {}", parent.display(), err))?;
    }
    let content = serde_json::to_string_pretty(value)
        .map_err(|err| format!("Failed to serialize JSON: {}", err))?;
    fs::write(path, format!("{}\n", content))
        .map_err(|err| format!("Failed to write {}: {}", path.display(), err))
}

fn toml_to_json(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(value) => JsonValue::String(value.clone()),
        TomlValue::Integer(value) => JsonValue::Number((*value).into()),
        TomlValue::Float(value) => JsonValue::Number(
            serde_json::Number::from_f64(*value).unwrap_or_else(|| 0.into()),
        ),
        TomlValue::Boolean(value) => JsonValue::Bool(*value),
        TomlValue::Datetime(value) => JsonValue::String(value.to_string()),
        TomlValue::Array(values) => {
            JsonValue::Array(values.iter().map(toml_to_json).collect())
        }
        TomlValue::Table(table) => {
            let mut map = JsonMap::new();
            for (key, value) in table {
                map.insert(key.clone(), toml_to_json(value));
            }
            JsonValue::Object(map)
        }
    }
}

fn json_to_toml(value: &JsonValue) -> Result<TomlValue, String> {
    match value {
        JsonValue::Null => Err("Null values are not supported".to_string()),
        JsonValue::Bool(value) => Ok(TomlValue::Boolean(*value)),
        JsonValue::Number(value) => {
            if let Some(int) = value.as_i64() {
                Ok(TomlValue::Integer(int))
            } else if let Some(float) = value.as_f64() {
                Ok(TomlValue::Float(float))
            } else {
                Err("Unsupported number".to_string())
            }
        }
        JsonValue::String(value) => Ok(TomlValue::String(value.clone())),
        JsonValue::Array(values) => {
            let mut array = Vec::new();
            for value in values {
                array.push(json_to_toml(value)?);
            }
            Ok(TomlValue::Array(array))
        }
        JsonValue::Object(values) => {
            let mut table = TomlMap::new();
            for (key, value) in values {
                table.insert(key.clone(), json_to_toml(value)?);
            }
            Ok(TomlValue::Table(table))
        }
    }
}

fn parse_mcp_json(input: &str) -> Result<HashMap<String, JsonValue>, String> {
    let value: JsonValue = serde_json::from_str(input)
        .map_err(|err| format!("Invalid MCP JSON: {}", err))?;
    let servers = value
        .get("mcpServers")
        .and_then(|item| item.as_object())
        .ok_or_else(|| "mcpServers object missing".to_string())?;

    let mut results = HashMap::new();
    for (id, config) in servers {
        results.insert(id.to_string(), config.clone());
    }

    Ok(results)
}

fn opencode_to_standard_config(config: &JsonValue) -> JsonValue {
    let Some(obj) = config.as_object() else {
        return config.clone();
    };
    let mut out = JsonMap::new();

    if let Some(url) = obj.get("url") {
        out.insert("url".to_string(), url.clone());
    }

    if let Some(command) = obj.get("command") {
        match command {
            JsonValue::Array(items) => {
                if let Some(first) = items.first().and_then(|item| item.as_str()) {
                    out.insert("command".to_string(), JsonValue::String(first.to_string()));
                    let args: Vec<JsonValue> = items
                        .iter()
                        .skip(1)
                        .filter_map(|item| item.as_str().map(|value| JsonValue::String(value.to_string())))
                        .collect();
                    if !args.is_empty() {
                        out.insert("args".to_string(), JsonValue::Array(args));
                    }
                }
            }
            JsonValue::String(value) => {
                out.insert("command".to_string(), JsonValue::String(value.to_string()));
            }
            _ => {}
        }
    }

    if let Some(enabled) = obj.get("enabled") {
        out.insert("enabled".to_string(), enabled.clone());
    }

    if let Some(server_type) = obj.get("type") {
        out.insert("type".to_string(), server_type.clone());
    }

    if let Some(environment) = obj.get("environment") {
        out.insert("env".to_string(), environment.clone());
    } else if let Some(env) = obj.get("env") {
        out.insert("env".to_string(), env.clone());
    }

    for (key, value) in obj {
        if ["command", "url", "enabled", "type", "env", "environment"].contains(&key.as_str()) {
            continue;
        }
        out.insert(key.clone(), value.clone());
    }

    JsonValue::Object(out)
}

fn standard_to_opencode_config(config: &JsonValue) -> Result<JsonValue, String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "MCP server config must be an object".to_string())?;
    let mut out = obj.clone();

    if let Some(command) = obj.get("command") {
        if let Some(command_str) = command.as_str() {
            let mut command_list = Vec::new();
            command_list.push(JsonValue::String(command_str.to_string()));
            if let Some(args) = obj.get("args").and_then(|value| value.as_array()) {
                for arg in args {
                    if let Some(value) = arg.as_str() {
                        command_list.push(JsonValue::String(value.to_string()));
                    }
                }
            }
            out.insert("command".to_string(), JsonValue::Array(command_list));
        }
        out.remove("args");
    }

    if let Some(env) = out.remove("env") {
        if !out.contains_key("environment") {
            out.insert("environment".to_string(), env);
        }
    }

    if !out.contains_key("type") {
        if out.contains_key("url") {
            out.insert("type".to_string(), JsonValue::String("remote".to_string()));
        } else if out.contains_key("command") {
            out.insert("type".to_string(), JsonValue::String("local".to_string()));
        }
    }

    Ok(JsonValue::Object(out))
}

fn read_mcp_servers(config: &McpSourceConfig, path: &Path) -> Result<Vec<McpServer>, String> {
    let mut servers = Vec::new();
    if !path.exists() {
        return Ok(servers);
    }

    match config.kind {
        McpKind::CodexToml => {
            let value = load_toml_value(path)?;
            if let Some(table) = value.get("mcp_servers").and_then(|item| item.as_table()) {
                for (id, server) in table {
                    servers.push(McpServer {
                        id: id.to_string(),
                        config: toml_to_json(server),
                    });
                }
            }
        }
        McpKind::ClaudeJson => {
            let value = load_json_value(path)?;
            if let Some(servers_value) = value.get("mcpServers") {
                if let Some(map) = servers_value.as_object() {
                    for (id, server) in map {
                        servers.push(McpServer {
                            id: id.to_string(),
                            config: server.clone(),
                        });
                    }
                }
            }
        }
        McpKind::OpenCodeJson => {
            let value = load_json_value(path)?;
            if let Some(servers_value) = value.get("mcp") {
                if let Some(map) = servers_value.as_object() {
                    for (id, server) in map {
                        servers.push(McpServer {
                            id: id.to_string(),
                            config: opencode_to_standard_config(server),
                        });
                    }
                }
            }
        }
    }

    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

fn upsert_mcp_servers(
    config: &McpSourceConfig,
    servers: HashMap<String, JsonValue>,
) -> Result<(), String> {
    match config.kind {
        McpKind::CodexToml => {
            let mut value = load_toml_value(&config.primary_path)?;
            let table = value
                .as_table_mut()
                .ok_or_else(|| "Invalid config format".to_string())?;
            let mcp_entry = table
                .entry("mcp_servers".to_string())
                .or_insert_with(|| TomlValue::Table(TomlMap::new()));
            let mcp_table = mcp_entry
                .as_table_mut()
                .ok_or_else(|| "Invalid mcp_servers format".to_string())?;

            for (id, config_value) in servers {
                let toml_value = json_to_toml(&config_value)?;
                mcp_table.insert(id, toml_value);
            }

            save_toml_value(&config.primary_path, &value)?;
        }
        McpKind::ClaudeJson => {
            let mut value = load_json_value(&config.primary_path)?;
            let root = value
                .as_object_mut()
                .ok_or_else(|| "Invalid JSON format".to_string())?;
            let servers_value = root
                .entry("mcpServers".to_string())
                .or_insert_with(|| JsonValue::Object(JsonMap::new()));
            let servers_map = servers_value
                .as_object_mut()
                .ok_or_else(|| "Invalid mcpServers format".to_string())?;

            for (id, config_value) in servers {
                servers_map.insert(id, config_value);
            }

            save_json_value(&config.primary_path, &value)?;
        }
        McpKind::OpenCodeJson => {
            let mut value = load_json_value(&config.primary_path)?;
            let root = value
                .as_object_mut()
                .ok_or_else(|| "Invalid JSON format".to_string())?;
            let mcp_value = root
                .entry("mcp".to_string())
                .or_insert_with(|| JsonValue::Object(JsonMap::new()));
            let mcp_map = mcp_value
                .as_object_mut()
                .ok_or_else(|| "Invalid mcp format".to_string())?;

            for (id, config_value) in servers {
                let converted = standard_to_opencode_config(&config_value)?;
                mcp_map.insert(id, converted);
            }

            save_json_value(&config.primary_path, &value)?;
        }
    }

    Ok(())
}

fn delete_mcp_server_for_source(config: &McpSourceConfig, server_id: &str) -> Result<(), String> {
    match config.kind {
        McpKind::CodexToml => {
            let mut value = load_toml_value(&config.primary_path)?;
            let table = value
                .as_table_mut()
                .ok_or_else(|| "Invalid config format".to_string())?;
            if let Some(mcp_table) = table
                .get_mut("mcp_servers")
                .and_then(|item| item.as_table_mut())
            {
                if mcp_table.remove(server_id).is_none() {
                    return Err("MCP server not found".to_string());
                }
            } else {
                return Err("No MCP servers configured".to_string());
            }
            save_toml_value(&config.primary_path, &value)?;
        }
        McpKind::ClaudeJson => {
            let mut value = load_json_value(&config.primary_path)?;
            let root = value
                .as_object_mut()
                .ok_or_else(|| "Invalid JSON format".to_string())?;
            let servers_value = root
                .get_mut("mcpServers")
                .and_then(|item| item.as_object_mut())
                .ok_or_else(|| "No mcpServers configured".to_string())?;

            if servers_value.remove(server_id).is_none() {
                return Err("MCP server not found".to_string());
            }
            save_json_value(&config.primary_path, &value)?;
        }
        McpKind::OpenCodeJson => {
            let mut value = load_json_value(&config.primary_path)?;
            let root = value
                .as_object_mut()
                .ok_or_else(|| "Invalid JSON format".to_string())?;
            let mcp_value = root
                .get_mut("mcp")
                .and_then(|item| item.as_object_mut())
                .ok_or_else(|| "No mcp configured".to_string())?;

            if mcp_value.remove(server_id).is_none() {
                return Err("MCP server not found".to_string());
            }
            save_json_value(&config.primary_path, &value)?;
        }
    }

    Ok(())
}

#[tauri::command]
fn list_skills() -> Result<Vec<SkillSource>, String> {
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let mut response = Vec::new();

    for source in sources {
        let installed = source.install_root.is_dir();
        if !installed {
            continue;
        }
        let root_exists = source.root.is_dir();
        let skills = if root_exists {
            read_skills(&source)
        } else {
            vec![]
        };
        response.push(SkillSource {
            id: source.id.to_string(),
            label: source.label.to_string(),
            root: source.root.display().to_string(),
            exists: root_exists,
            skills,
        });
    }

    Ok(response)
}

#[tauri::command]
fn list_skill_tree(payload: SkillTreeInput) -> Result<SkillTreeNode, String> {
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let source = sources
        .iter()
        .find(|source| source.id == payload.source_id)
        .ok_or_else(|| "Unknown skill source".to_string())?;

    let skill_dir = source.root.join(&payload.skill_id);
    if !skill_dir.exists() {
        return Err("Skill not found".to_string());
    }

    let root_canon = fs::canonicalize(&source.root)
        .map_err(|err| format!("Failed to resolve root: {}", err))?;
    let skill_canon = fs::canonicalize(&skill_dir)
        .map_err(|err| format!("Failed to resolve skill: {}", err))?;
    if !skill_canon.starts_with(&root_canon) {
        return Err("Refusing to read outside agent root".to_string());
    }

    build_skill_tree(&skill_dir)
}

#[tauri::command]
fn install_skill_from_url(payload: InstallSkillInput) -> Result<SkillItem, String> {
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let source = sources
        .iter()
        .find(|source| source.id == payload.source_id)
        .ok_or_else(|| "Unknown skill source".to_string())?;

    fs::create_dir_all(&source.root)
        .map_err(|err| format!("Failed to create {}: {}", source.root.display(), err))?;

    let github_location = parse_github_location(&payload.url).ok();
    let github_agent = github_location
        .as_ref()
        .map(|_| ureq::AgentBuilder::new().user_agent("Ananke/0.1").build());
    let github_branches = match (github_location.as_ref(), github_agent.as_ref()) {
        (Some(location), Some(agent)) => Some(github_branch_candidates(agent, location)),
        _ => None,
    };
    let mut content = None;
    let mut core_file_name = None;
    let mut last_error = None;
    let mut selected_branch = None;

    'core_lookup: for file_name in &source.core_files {
        if let (Some(location), Some(agent), Some(branches)) =
            (github_location.as_ref(), github_agent.as_ref(), github_branches.as_ref())
        {
            for branch in branches {
                let path = github_file_path(location, file_name);
                match fetch_github_file_content(
                    agent,
                    &location.owner,
                    &location.repo,
                    &path,
                    branch,
                ) {
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Ok(body) => {
                            content = Some(body);
                            core_file_name = Some(file_name.to_string());
                            selected_branch = Some(branch.to_string());
                            break 'core_lookup;
                        }
                        Err(err) => {
                            last_error =
                                Some(format!("GitHub file is not UTF-8: {}", err));
                        }
                    },
                    Err(err) => {
                        last_error = Some(err);
                    }
                }
            }
        } else {
            let candidates = parse_skill_urls(&payload.url, file_name)?;
            match fetch_skill_content(candidates) {
                Ok(body) => {
                    content = Some(body);
                    core_file_name = Some(file_name.to_string());
                    break;
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }
    }

    let content = content.ok_or_else(|| {
        last_error.unwrap_or_else(|| "Unable to download skill file".to_string())
    })?;
    let core_file_name = core_file_name.ok_or_else(|| "Missing core file".to_string())?;
    let is_markdown = core_file_name.ends_with(".md");
    let (metadata, _) = if is_markdown {
        parse_frontmatter(&content)
    } else {
        (HashMap::new(), content.clone())
    };

    let name = metadata
        .get("name")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback_name_from_url(&payload.url, &core_file_name));

    let base_slug = slugify(&name);
    let mut skill_dir = source.root.join(&base_slug);
    if skill_dir.exists() {
        let mut suffix = 1;
        loop {
            let candidate = source.root.join(format!("{}-{}", base_slug, suffix));
            if !candidate.exists() {
                skill_dir = candidate;
                break;
            }
            suffix += 1;
        }
    }

    fs::create_dir_all(&skill_dir)
        .map_err(|err| format!("Failed to create {}: {}", skill_dir.display(), err))?;
    let core_path = skill_dir.join(&core_file_name);
    if let (Some(location), Some(agent), Some(mut branches)) =
        (github_location, github_agent, github_branches)
    {
        if let Some(branch) = selected_branch {
            branches.retain(|item| item != &branch);
            branches.insert(0, branch);
        }
        let mut last_download_error = None;
        let mut downloaded = false;
        for branch in branches {
            match download_github_directory(&agent, &location, &branch, &skill_dir) {
                Ok(_) => {
                    downloaded = true;
                    break;
                }
                Err(err) => {
                    last_download_error = Some(err);
                }
            }
        }
        if !downloaded {
            return Err(last_download_error
                .unwrap_or_else(|| "Unable to download GitHub directory".to_string()));
        }
    }

    write_skill_source_url(&skill_dir, &payload.url)?;
    fs::write(&core_path, content)
        .map_err(|err| format!("Failed to write {}: {}", core_file_name, err))?;

    load_skill(&skill_dir, &core_path, &core_file_name, source)
}

#[tauri::command]
fn sync_skill_from_url(payload: SyncSkillInput) -> Result<SkillItem, String> {
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let source = sources
        .iter()
        .find(|source| source.id == payload.source_id)
        .ok_or_else(|| "Unknown skill source".to_string())?;

    let skill_dir = source.root.join(&payload.skill_id);
    if !skill_dir.exists() {
        return Err("Skill not found".to_string());
    }

    let root_canon = fs::canonicalize(&source.root)
        .map_err(|err| format!("Failed to resolve root: {}", err))?;
    let skill_canon = fs::canonicalize(&skill_dir)
        .map_err(|err| format!("Failed to resolve skill: {}", err))?;
    if !skill_canon.starts_with(&root_canon) {
        return Err("Refusing to sync outside agent root".to_string());
    }

    let (core_file_path, core_file_name) = find_core_file(&skill_dir, &source.core_files)
        .ok_or_else(|| "Missing core file".to_string())?;

    if let Ok(location) = parse_github_location(&payload.url) {
        let agent = ureq::AgentBuilder::new()
            .user_agent("Ananke/0.1")
            .build();
        let mut branches = github_branch_candidates(&agent, &location);
        let mut last_download_error = None;
        let mut confirmed_branch = None;
        for branch in &branches {
            match download_github_directory(&agent, &location, branch, &skill_dir) {
                Ok(_) => {
                    confirmed_branch = Some(branch.to_string());
                    break;
                }
                Err(err) => {
                    last_download_error = Some(err);
                }
            }
        }
        if confirmed_branch.is_none() {
            return Err(last_download_error
                .unwrap_or_else(|| "Unable to download GitHub directory".to_string()));
        }
        let branch = confirmed_branch.unwrap_or_else(|| branches.remove(0));
        let path = github_file_path(&location, &core_file_name);
        let content = fetch_github_file_content(
            &agent,
            &location.owner,
            &location.repo,
            &path,
            &branch,
        )
        .and_then(|bytes| {
            String::from_utf8(bytes)
                .map_err(|err| format!("GitHub file is not UTF-8: {}", err))
        })?;
        write_skill_source_url(&skill_dir, &payload.url)?;
        fs::write(&core_file_path, content)
            .map_err(|err| format!("Failed to write {}: {}", core_file_name, err))?;
    } else {
        let candidates = parse_skill_urls(&payload.url, &core_file_name)?;
        let content = fetch_skill_content(candidates)?;
        write_skill_source_url(&skill_dir, &payload.url)?;
        fs::write(&core_file_path, content)
            .map_err(|err| format!("Failed to write {}: {}", core_file_name, err))?;
    }

    load_skill(&skill_dir, &core_file_path, &core_file_name, source)
}

#[tauri::command]
fn delete_skill(payload: DeleteSkillInput) -> Result<(), String> {
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let source = sources
        .iter()
        .find(|source| source.id == payload.source_id)
        .ok_or_else(|| "Unknown skill source".to_string())?;

    let skill_dir = source.root.join(&payload.skill_id);
    if !skill_dir.exists() {
        return Err("Skill not found".to_string());
    }

    let root_canon = fs::canonicalize(&source.root)
        .map_err(|err| format!("Failed to resolve root: {}", err))?;
    let skill_canon = fs::canonicalize(&skill_dir)
        .map_err(|err| format!("Failed to resolve skill: {}", err))?;

    if !skill_canon.starts_with(&root_canon) {
        return Err("Refusing to delete outside agent root".to_string());
    }

    fs::remove_dir_all(&skill_dir)
        .map_err(|err| format!("Failed to delete skill: {}", err))?;
    Ok(())
}

#[tauri::command]
fn sync_skills_from_agent(payload: SyncAgentsInput) -> Result<SyncResult, String> {
    if payload.source_id == payload.target_id {
        return Err("Source and target must be different".to_string());
    }
    let home = resolve_home()?;
    let sources = source_configs(&home);
    let source = sources
        .iter()
        .find(|source| source.id == payload.source_id)
        .ok_or_else(|| "Unknown skill source".to_string())?;
    let target = sources
        .iter()
        .find(|source| source.id == payload.target_id)
        .ok_or_else(|| "Unknown target source".to_string())?;

    if !source.root.is_dir() {
        return Err("Source skills directory missing".to_string());
    }
    fs::create_dir_all(&target.root)
        .map_err(|err| format!("Failed to create {}: {}", target.root.display(), err))?;

    let source_root = fs::canonicalize(&source.root)
        .map_err(|err| format!("Failed to resolve source root: {}", err))?;
    let skills = read_skills(source);
    let mut added = 0;
    let mut skipped = 0;

    for skill in skills {
        let skill_dir = source.root.join(&skill.id);
        let skill_canon = fs::canonicalize(&skill_dir)
            .map_err(|err| format!("Failed to resolve skill: {}", err))?;
        if !skill_canon.starts_with(&source_root) {
            return Err("Refusing to copy outside agent root".to_string());
        }

        let target_dir = target.root.join(&skill.id);
        if target_dir.exists() {
            skipped += 1;
            continue;
        }
        copy_dir_recursive(&skill_dir, &target_dir)?;
        added += 1;
    }

    Ok(SyncResult { added, skipped })
}

#[tauri::command]
fn list_mcp_sources() -> Result<Vec<McpSource>, String> {
    let home = resolve_home()?;
    let configs = mcp_source_configs(&home);
    let mut response = Vec::new();

    for config in configs {
        if !config.install_root.is_dir() {
            continue;
        }
        let path = resolve_read_path(&config);
        let exists = path.exists();
        let servers = if exists {
            read_mcp_servers(&config, &path)?
        } else {
            vec![]
        };

        response.push(McpSource {
            id: config.id.to_string(),
            label: config.label.to_string(),
            path: path.display().to_string(),
            format: config.format.to_string(),
            exists,
            servers,
        });
    }

    Ok(response)
}

#[tauri::command]
fn sync_mcp_from_agent(payload: SyncAgentsInput) -> Result<SyncResult, String> {
    if payload.source_id == payload.target_id {
        return Err("Source and target must be different".to_string());
    }
    let home = resolve_home()?;
    let configs = mcp_source_configs(&home);
    let source = configs
        .iter()
        .find(|config| config.id == payload.source_id)
        .ok_or_else(|| "Unknown MCP source".to_string())?;
    let target = configs
        .iter()
        .find(|config| config.id == payload.target_id)
        .ok_or_else(|| "Unknown MCP target".to_string())?;

    let source_path = resolve_read_path(source);
    let source_servers = read_mcp_servers(source, &source_path)?;

    let target_path = resolve_read_path(target);
    let target_servers = if target_path.exists() {
        read_mcp_servers(target, &target_path)?
    } else {
        Vec::new()
    };
    let existing_ids: HashSet<String> =
        target_servers.into_iter().map(|server| server.id).collect();

    let mut added = 0;
    let mut skipped = 0;
    let mut to_insert = HashMap::new();

    for server in source_servers {
        if existing_ids.contains(&server.id) {
            skipped += 1;
            continue;
        }
        to_insert.insert(server.id.clone(), server.config);
        added += 1;
    }

    if !to_insert.is_empty() {
        upsert_mcp_servers(target, to_insert)?;
    }

    Ok(SyncResult { added, skipped })
}

#[tauri::command]
fn upsert_mcp_server_json(payload: UpsertMcpJsonInput) -> Result<(), String> {
    let home = resolve_home()?;
    let configs = mcp_source_configs(&home);
    let config = configs
        .iter()
        .find(|config| config.id == payload.source_id)
        .ok_or_else(|| "Unknown MCP source".to_string())?;

    let servers = parse_mcp_json(&payload.json)?;
    upsert_mcp_servers(config, servers)
}

#[tauri::command]
fn delete_mcp_server(payload: DeleteMcpInput) -> Result<(), String> {
    let home = resolve_home()?;
    let configs = mcp_source_configs(&home);
    let config = configs
        .iter()
        .find(|config| config.id == payload.source_id)
        .ok_or_else(|| "Unknown MCP source".to_string())?;

    delete_mcp_server_for_source(config, &payload.id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_skills,
            list_skill_tree,
            install_skill_from_url,
            sync_skill_from_url,
            delete_skill,
            sync_skills_from_agent,
            list_mcp_sources,
            sync_mcp_from_agent,
            upsert_mcp_server_json,
            delete_mcp_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
