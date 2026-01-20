import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

type Skill = {
  id: string;
  name: string;
  description: string;
  path: string;
  coreFile: string;
  coreFilePath: string;
  sourceUrl?: string | null;
  sourceId: string;
  metadata: Record<string, string>;
  body: string;
  lastModified?: number;
};

type SkillSource = {
  id: string;
  label: string;
  root: string;
  exists: boolean;
  skills: Skill[];
};

type SkillTreeNode = {
  name: string;
  path: string;
  kind: "file" | "dir" | "link";
  children: SkillTreeNode[];
};

type McpServer = {
  id: string;
  config: Record<string, unknown>;
};

type McpSource = {
  id: string;
  label: string;
  path: string;
  format: "toml" | "json";
  exists: boolean;
  servers: McpServer[];
};

type ToastTone = "success" | "error" | "info";

type ToastState = {
  message: string;
  tone: ToastTone;
} | null;

type SkillForm = {
  sourceId: string;
  url: string;
};

type McpForm = {
  sourceId: string;
  json: string;
};

type Locale = "en" | "zh";

const translations = {
  en: {
    tagline: "One Place for Skills & MCP.",
    viewSkills: "Skills",
    viewMcp: "MCP",
    installSkill: "Install skill",
    skillCenter: "Skill Center",
    refresh: "Refresh",
    addMcpServer: "Add MCP server",
    aiAgents: "AI Coding Agents",
    pickScope: "Pick a scope",
    mcpScopes: "MCP scopes",
    allAgents: "All Agents",
    unifiedView: "Unified view",
    statusReady: "Ready",
    statusMissing: "Missing",
    installedSkills: "Installed Skills",
    allAgentsLabel: "All agents",
    indexingSkills: "Indexing skills...",
    noSkillsMatch: "No skills match this filter.",
    noDescription: "No description",
    skillDetails: "Skill Details",
    fullInstructions: "Full instructions",
    selectSkillInspect: "Select a skill to inspect.",
    labelPath: "Path",
    labelLastModified: "Last modified",
    labelAgentRoot: "Agent root",
    directoryTree: "Directory tree",
    loadingTree: "Loading tree...",
    noTreeData: "No tree data.",
    skillContent: "SKILL.md content",
    syncLatest: "Sync latest",
    syncing: "Syncing...",
    deleteSkill: "Delete skill",
    mcpServers: "MCP Servers",
    selectAgent: "Select an agent",
    loadingMcpServers: "Loading MCP servers...",
    selectAgentToViewMcp: "Select an agent to view MCP.",
    noMcpConfigured: "No MCP servers configured.",
    mcpDetails: "MCP Details",
    jsonFormat: "JSON format",
    selectMcpServer: "Select an MCP server.",
    unknownAgent: "Unknown agent",
    agentLabel: "Agent",
    mcpJsonLabel: "mcpServers JSON",
    editJson: "Edit JSON",
    deleteServer: "Delete server",
    installSkillTitle: "Install skill from GitHub",
    installSkillHint: "Pick an agent and provide a GitHub directory URL.",
    close: "Close",
    targetAgent: "Target agent",
    githubUrl: "GitHub directory URL",
    githubUrlPlaceholder: "https://github.com/org/repo/tree/main/skills/example",
    confirmDeleteSkillTitle: "Delete skill",
    confirmDeleteServerTitle: "Delete MCP server",
    irreversible: "This cannot be undone.",
    labelSkill: "Skill",
    labelServer: "Server",
    cancel: "Cancel",
    registerMcpTitle: "Register MCP server",
    registerMcpHint: "Paste MCP JSON for the selected agent.",
    mcpJson: "MCP JSON",
    saveMcp: "Save MCP",
    footerCopyright: "Copyright {year} Ananke",
    footerRegistry: "SKILL.md registry",
    countSkills: "{count} skills",
    countMcpServers: "{count} MCP servers",
    agentMeta: "Agent: {agent}",
    unselected: "Unselected",
    urlMeta: "url: {url}",
    cmdMeta: "cmd: {command}",
    fieldCount: "{count} fields",
    argsCount: "{count} args",
    openFailed: "Open failed: {error}",
    githubUrlRequired: "GitHub URL is required.",
    skillInstalled: "Skill installed.",
    installFailed: "Install failed: {error}",
    skillSynced: "Skill synced.",
    syncFailed: "Sync failed: {error}",
    skillDeleted: "Skill deleted.",
    deleteFailed: "Delete failed: {error}",
    agentsRefreshed: "Agents refreshed.",
    mcpRefreshed: "MCP servers refreshed.",
    invalidJson: "Invalid JSON format.",
    jsonMustBeObject: "JSON must be an object.",
    jsonMustIncludeMcp: "JSON must include mcpServers object.",
    mcpSaved: "MCP server saved.",
    saveFailed: "Save failed: {error}",
    mcpDeleted: "MCP server deleted.",
    notTracked: "Not tracked",
    treeDir: "DIR",
    treeLink: "LINK",
    treeFile: "FILE",
    language: "Language",
    english: "English",
    chinese: "中文",
  },
  zh: {
    tagline: "Skill 与 MCP 的可视化管理工具",
    viewSkills: "Skill",
    viewMcp: "MCP",
    installSkill: "安装Skill",
    skillCenter: "Skill中心",
    refresh: "刷新",
    addMcpServer: "添加 MCP 服务",
    aiAgents: "AI 编码Agent",
    pickScope: "选择范围",
    mcpScopes: "MCP 范围",
    allAgents: "全部Agent",
    unifiedView: "统一视图",
    statusReady: "可用",
    statusMissing: "缺失",
    installedSkills: "已安装Skills",
    allAgentsLabel: "全部Agent",
    indexingSkills: "正在索引Skill...",
    noSkillsMatch: "没有匹配的Skill。",
    noDescription: "暂无描述",
    skillDetails: "Skill详情",
    fullInstructions: "完整说明",
    selectSkillInspect: "选择一个Skill查看详情。",
    labelPath: "路径",
    labelLastModified: "最近更新",
    labelAgentRoot: "Agent目录",
    directoryTree: "目录结构",
    loadingTree: "正在加载目录...",
    noTreeData: "没有目录数据。",
    skillContent: "SKILL.md 内容",
    syncLatest: "同步最新",
    syncing: "同步中...",
    deleteSkill: "删除Skill",
    mcpServers: "MCP 服务",
    selectAgent: "选择Agent",
    loadingMcpServers: "正在加载 MCP 服务...",
    selectAgentToViewMcp: "选择Agent以查看 MCP。",
    noMcpConfigured: "暂无 MCP 服务。",
    mcpDetails: "MCP 详情",
    jsonFormat: "JSON 格式",
    selectMcpServer: "选择一个 MCP 服务。",
    unknownAgent: "未知Agent",
    agentLabel: "Agent",
    mcpJsonLabel: "mcpServers JSON",
    editJson: "编辑 JSON",
    deleteServer: "删除服务",
    installSkillTitle: "从 GitHub 安装Skill",
    installSkillHint: "选择Agent并填写 GitHub 目录链接。",
    close: "关闭",
    targetAgent: "目标Agent",
    githubUrl: "GitHub 目录链接",
    githubUrlPlaceholder: "https://github.com/org/repo/tree/main/skills/example",
    confirmDeleteSkillTitle: "删除Skill",
    confirmDeleteServerTitle: "删除 MCP 服务",
    irreversible: "此操作无法撤销。",
    labelSkill: "Skill",
    labelServer: "服务",
    cancel: "取消",
    registerMcpTitle: "注册 MCP 服务",
    registerMcpHint: "粘贴所选Agent的 MCP JSON。",
    mcpJson: "MCP JSON",
    saveMcp: "保存 MCP",
    footerCopyright: "版权所有 {year} Ananke",
    footerRegistry: "SKILL.md 注册中心",
    countSkills: "{count} 个Skill",
    countMcpServers: "{count} 个 MCP 服务",
    agentMeta: "Agent：{agent}",
    unselected: "未选择",
    urlMeta: "URL：{url}",
    cmdMeta: "命令：{command}",
    fieldCount: "{count} 个字段",
    argsCount: "{count} 个参数",
    openFailed: "打开失败：{error}",
    githubUrlRequired: "需要填写 GitHub 链接。",
    skillInstalled: "Skill已安装。",
    installFailed: "安装失败：{error}",
    skillSynced: "Skill已同步。",
    syncFailed: "同步失败：{error}",
    skillDeleted: "Skill已删除。",
    deleteFailed: "删除失败：{error}",
    agentsRefreshed: "Agent列表已刷新。",
    mcpRefreshed: "MCP 服务已刷新。",
    invalidJson: "JSON 格式无效。",
    jsonMustBeObject: "JSON 必须是对象。",
    jsonMustIncludeMcp: "JSON 必须包含 mcpServers 对象。",
    mcpSaved: "MCP 服务已保存。",
    saveFailed: "保存失败：{error}",
    mcpDeleted: "MCP 服务已删除。",
    notTracked: "未记录",
    treeDir: "目录",
    treeLink: "链接",
    treeFile: "文件",
    language: "语言",
    english: "English",
    chinese: "中文",
  },
} as const;

type TranslationKey = keyof typeof translations.en;
type TranslationVars = Record<string, string | number>;

const resolveLocale = (): Locale => {
  if (typeof window === "undefined") return "en";
  const stored = window.localStorage.getItem("ananke.locale");
  if (stored === "en" || stored === "zh") return stored;
  const languages = navigator.languages ?? [navigator.language];
  const isZh = languages.some((lang) => lang.toLowerCase().startsWith("zh"));
  return isZh ? "zh" : "en";
};

type DeleteIntent =
  | {
      kind: "skill";
      sourceId: string;
      skillId: string;
      name: string;
    }
  | {
      kind: "mcp";
      sourceId: string;
      id: string;
      name: string;
    };

type SkillWithSource = Skill & {
  sourceLabel: string;
  sourceRoot: string;
};

const sourcePalette: Record<
  string,
  { accent: string; soft: string; ink: string }
> = {
  all: { accent: "#1f1a16", soft: "#f5efe7", ink: "#1f1a16" },
  claude: { accent: "#3a8b6b", soft: "#d7efe6", ink: "#1f1a16" },
  codex: { accent: "#2b5da8", soft: "#dde7f7", ink: "#1f1a16" },
  opencode: { accent: "#9a7a2c", soft: "#f3ead3", ink: "#1f1a16" },
  roo: { accent: "#566b2f", soft: "#e6edd9", ink: "#1f1a16" },
  copilot: { accent: "#0f6b57", soft: "#d8efe9", ink: "#1f1a16" },
  cursor: { accent: "#b24a2d", soft: "#f6e1da", ink: "#1f1a16" },
  gemini: { accent: "#b9782a", soft: "#f4e7d6", ink: "#1f1a16" },
  trae: { accent: "#b2416e", soft: "#f3dbe7", ink: "#1f1a16" },
  goose: { accent: "#2d6d7a", soft: "#d7ecf1", ink: "#1f1a16" },
  standard: { accent: "#5b5248", soft: "#eee7dc", ink: "#1f1a16" },
  antigravity: { accent: "#7a5230", soft: "#f1e4d7", ink: "#1f1a16" },
  kiro: { accent: "#517d3b", soft: "#e1efda", ink: "#1f1a16" },
  qoder: { accent: "#3b4a78", soft: "#dfe4f1", ink: "#1f1a16" },
  codebuddy: { accent: "#7a2d2d", soft: "#f2dbdb", ink: "#1f1a16" },
};

const paletteForSource = (sourceId: string) => {
  const base = sourceId.split("-")[0];
  return sourcePalette[base] || sourcePalette.all;
};

const defaultSkillForm: SkillForm = {
  sourceId: "codex-user",
  url: "",
};

const defaultMcpJson = `{
  "mcpServers": {
    "server-id": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
    }
  }
}`;

const defaultMcpForm: McpForm = {
  sourceId: "codex",
  json: defaultMcpJson,
};

const shortenPath = (value: string) => {
  if (!value) return "";
  if (value.length < 40) return value;
  return `${value.slice(0, 18)}...${value.slice(-18)}`;
};

const buildMcpJson = (server: McpServer) =>
  JSON.stringify({ mcpServers: { [server.id]: server.config } }, null, 2);

function App() {
  const [view, setView] = useState<"skills" | "mcp">("skills");
  const [sources, setSources] = useState<SkillSource[]>([]);
  const [selectedSource, setSelectedSource] = useState("all");
  const [selectedSkillKey, setSelectedSkillKey] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<ToastState>(null);
  const [showAddSkill, setShowAddSkill] = useState(false);
  const [skillForm, setSkillForm] = useState<SkillForm>(defaultSkillForm);
  const [deleteIntent, setDeleteIntent] = useState<DeleteIntent | null>(null);
  const [syncLoading, setSyncLoading] = useState(false);
  const toastTimer = useRef<number | null>(null);

  const [skillTree, setSkillTree] = useState<SkillTreeNode | null>(null);
  const [skillTreeLoading, setSkillTreeLoading] = useState(false);
  const [skillTreeError, setSkillTreeError] = useState<string | null>(null);

  const [mcpSources, setMcpSources] = useState<McpSource[]>([]);
  const [mcpLoading, setMcpLoading] = useState(true);
  const [mcpError, setMcpError] = useState<string | null>(null);
  const [selectedMcpSource, setSelectedMcpSource] = useState("codex");
  const [selectedMcpId, setSelectedMcpId] = useState<string | null>(null);
  const [showAddMcp, setShowAddMcp] = useState(false);
  const [mcpForm, setMcpForm] = useState<McpForm>(defaultMcpForm);
  const [locale, setLocale] = useState<Locale>(() => resolveLocale());

  useEffect(() => {
    window.localStorage.setItem("ananke.locale", locale);
  }, [locale]);

  const t = (key: TranslationKey, vars?: TranslationVars) => {
    const template = translations[locale][key];
    if (!vars) return template;
    return template.replace(/\{(\w+)\}/g, (match, token) => {
      const value = vars[token];
      return value === undefined ? match : String(value);
    });
  };

  const localeTag = locale === "zh" ? "zh-CN" : "en-US";
  const formatDate = (value?: number) => {
    if (!value) return t("notTracked");
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return t("notTracked");
    return new Intl.DateTimeFormat(localeTag, {
      year: "numeric",
      month: "short",
      day: "2-digit",
    }).format(date);
  };

  const skillTreeKindLabel = (kind: SkillTreeNode["kind"]) => {
    if (kind === "dir") return t("treeDir");
    if (kind === "link") return t("treeLink");
    return t("treeFile");
  };

  const loadSources = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<SkillSource[]>("list_skills");
      setSources(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const loadMcp = async () => {
    setMcpLoading(true);
    setMcpError(null);
    try {
      const result = await invoke<McpSource[]>("list_mcp_sources");
      setMcpSources(result);
    } catch (err) {
      setMcpError(String(err));
    } finally {
      setMcpLoading(false);
    }
  };

  useEffect(() => {
    loadSources();
    loadMcp();
  }, []);

  useEffect(() => {
    if (sources.length === 0) return;
    if (!sources.some((source) => source.id === skillForm.sourceId)) {
      setSkillForm((current) => ({
        ...current,
        sourceId: sources[0].id,
      }));
    }
  }, [sources, skillForm.sourceId]);

  useEffect(() => {
    if (selectedSource === "all") return;
    if (!sources.some((source) => source.id === selectedSource)) {
      setSelectedSource("all");
    }
  }, [sources, selectedSource]);

  useEffect(() => {
    if (!selectedSkillKey) {
      setSkillTree(null);
      return;
    }
  }, [selectedSkillKey]);

  const allSkills = useMemo<SkillWithSource[]>(() => {
    return sources.flatMap((source) =>
      source.skills.map((skill) => ({
        ...skill,
        sourceLabel: source.label,
        sourceRoot: source.root,
      })),
    );
  }, [sources]);

  const visibleSkills = useMemo(() => {
    return selectedSource === "all"
      ? allSkills
      : allSkills.filter((skill) => skill.sourceId === selectedSource);
  }, [allSkills, selectedSource]);

  const selectedSkill = useMemo(() => {
    if (!selectedSkillKey) return null;
    return (
      allSkills.find(
        (skill) => `${skill.sourceId}:${skill.id}` === selectedSkillKey,
      ) || null
    );
  }, [allSkills, selectedSkillKey]);

  const totalSkills = allSkills.length;

  useEffect(() => {
    if (!selectedSkill) {
      setSkillTree(null);
      setSkillTreeLoading(false);
      setSkillTreeError(null);
      return;
    }

    let cancelled = false;
    setSkillTreeLoading(true);
    setSkillTreeError(null);

    invoke<SkillTreeNode>("list_skill_tree", {
      payload: {
        sourceId: selectedSkill.sourceId,
        skillId: selectedSkill.id,
      },
    })
      .then((tree) => {
        if (!cancelled) {
          setSkillTree(tree);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setSkillTreeError(String(err));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setSkillTreeLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedSkill?.sourceId, selectedSkill?.id]);

  const activeMcpSource = useMemo(() => {
    return mcpSources.find((source) => source.id === selectedMcpSource) || null;
  }, [mcpSources, selectedMcpSource]);

  useEffect(() => {
    if (mcpSources.length === 0) return;
    if (!mcpSources.some((source) => source.id === selectedMcpSource)) {
      setSelectedMcpSource(mcpSources[0].id);
    }
  }, [mcpSources, selectedMcpSource]);

  useEffect(() => {
    setSelectedMcpId(null);
  }, [selectedMcpSource]);

  const selectedMcp = useMemo(() => {
    if (!selectedMcpId || !activeMcpSource) return null;
    return (
      activeMcpSource.servers.find((server) => server.id === selectedMcpId) ||
      null
    );
  }, [activeMcpSource, selectedMcpId]);

  const selectedMcpJson = useMemo(() => {
    if (!selectedMcp) return "";
    return buildMcpJson(selectedMcp);
  }, [selectedMcp]);

  const totalMcpServers = useMemo(() => {
    return mcpSources.reduce((sum, source) => sum + source.servers.length, 0);
  }, [mcpSources]);

  const handleSelectSkill = (skill: SkillWithSource) => {
    setSelectedSkillKey(`${skill.sourceId}:${skill.id}`);
  };

  const showToast = (message: string, tone: ToastTone) => {
    setToast({ message, tone });
    if (toastTimer.current) {
      window.clearTimeout(toastTimer.current);
    }
    toastTimer.current = window.setTimeout(() => {
      setToast(null);
    }, 2400);
  };

  const handleOpenSkillCenter = async () => {
    try {
      await openUrl("https://skill.extrachatgpt.com/");
    } catch (err) {
      showToast(t("openFailed", { error: String(err) }), "error");
    }
  };

  const handleOpenAddSkill = () => {
    const preferred =
      selectedSource !== "all" &&
      sources.some((source) => source.id === selectedSource)
        ? selectedSource
        : sources[0]?.id;
    if (preferred && preferred !== skillForm.sourceId) {
      setSkillForm((current) => ({
        ...current,
        sourceId: preferred,
      }));
    }
    setShowAddSkill(true);
  };

  const handleInstallSkill = async () => {
    if (!skillForm.url.trim()) {
      showToast(t("githubUrlRequired"), "error");
      return;
    }
    try {
      const created = await invoke<Skill>("install_skill_from_url", {
        payload: {
          sourceId: skillForm.sourceId,
          url: skillForm.url.trim(),
        },
      });
      await loadSources();
      setSelectedSkillKey(`${created.sourceId}:${created.id}`);
      setSkillForm(defaultSkillForm);
      setShowAddSkill(false);
      showToast(t("skillInstalled"), "success");
    } catch (err) {
      showToast(t("installFailed", { error: String(err) }), "error");
    }
  };

  const handleSyncSkill = async () => {
    if (!selectedSkill?.sourceUrl || syncLoading) return;
    const skillKey = `${selectedSkill.sourceId}:${selectedSkill.id}`;
    try {
      setSyncLoading(true);
      await invoke<Skill>("sync_skill_from_url", {
        payload: {
          sourceId: selectedSkill.sourceId,
          skillId: selectedSkill.id,
          url: selectedSkill.sourceUrl,
        },
      });
      await loadSources();
      setSelectedSkillKey(skillKey);
      showToast(t("skillSynced"), "success");
    } catch (err) {
      showToast(t("syncFailed", { error: String(err) }), "error");
    } finally {
      setSyncLoading(false);
    }
  };

  const handleRequestDeleteSkill = () => {
    if (!selectedSkill) return;
    setDeleteIntent({
      kind: "skill",
      sourceId: selectedSkill.sourceId,
      skillId: selectedSkill.id,
      name: selectedSkill.name,
    });
  };

  const handleDeleteSkill = async (sourceId: string, skillId: string) => {
    try {
      await invoke("delete_skill", {
        payload: {
          sourceId,
          skillId,
        },
      });
      setSelectedSkillKey(null);
      await loadSources();
      showToast(t("skillDeleted"), "success");
    } catch (err) {
      showToast(t("deleteFailed", { error: String(err) }), "error");
    }
  };

  const handleRefreshSkills = async () => {
    await loadSources();
    showToast(t("agentsRefreshed"), "info");
  };

  const handleRefreshMcp = async () => {
    await loadMcp();
    showToast(t("mcpRefreshed"), "info");
  };

  const handleOpenAddMcp = () => {
    setMcpForm({
      sourceId: selectedMcpSource || "codex",
      json: defaultMcpJson,
    });
    setShowAddMcp(true);
  };

  const handleSaveMcp = async () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(mcpForm.json);
    } catch (err) {
      showToast(t("invalidJson"), "error");
      return;
    }

    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      showToast(t("jsonMustBeObject"), "error");
      return;
    }

    const servers = (parsed as { mcpServers?: unknown }).mcpServers;
    if (!servers || typeof servers !== "object" || Array.isArray(servers)) {
      showToast(t("jsonMustIncludeMcp"), "error");
      return;
    }

    try {
      await invoke("upsert_mcp_server_json", {
        payload: {
          sourceId: mcpForm.sourceId,
          json: mcpForm.json,
        },
      });
      await loadMcp();
      setShowAddMcp(false);
      setMcpForm(defaultMcpForm);
      showToast(t("mcpSaved"), "success");
    } catch (err) {
      showToast(t("saveFailed", { error: String(err) }), "error");
    }
  };

  const handleRequestDeleteMcp = () => {
    if (!selectedMcp || !activeMcpSource) return;
    setDeleteIntent({
      kind: "mcp",
      sourceId: activeMcpSource.id,
      id: selectedMcp.id,
      name: selectedMcp.id,
    });
  };

  const handleDeleteMcp = async (sourceId: string, id: string) => {
    try {
      await invoke("delete_mcp_server", {
        payload: {
          sourceId,
          id,
        },
      });
      setSelectedMcpId(null);
      await loadMcp();
      showToast(t("mcpDeleted"), "success");
    } catch (err) {
      showToast(t("deleteFailed", { error: String(err) }), "error");
    }
  };

  const handleConfirmDelete = async () => {
    if (!deleteIntent) return;
    const intent = deleteIntent;
    setDeleteIntent(null);
    if (intent.kind === "skill") {
      await handleDeleteSkill(intent.sourceId, intent.skillId);
      return;
    }
    await handleDeleteMcp(intent.sourceId, intent.id);
  };

  const handleEditMcp = () => {
    if (!selectedMcp || !activeMcpSource) return;
    setMcpForm({
      sourceId: activeMcpSource.id,
      json: buildMcpJson(selectedMcp),
    });
    setShowAddMcp(true);
  };

  const sourceCount = (sourceId: string) =>
    sourceId === "all"
      ? totalSkills
      : sources.find((source) => source.id === sourceId)?.skills.length || 0;

  const renderTreeNode = (node: SkillTreeNode) => {
    return (
      <div key={node.path} className={`tree-node ${node.kind}`}>
        <div className="tree-row">
          <span className="tree-kind">{skillTreeKindLabel(node.kind)}</span>
          <span className="tree-name">{node.name}</span>
        </div>
        {node.children.length > 0 && (
          <div className="tree-children">
            {node.children.map((child) => renderTreeNode(child))}
          </div>
        )}
      </div>
    );
  };

  const selectedMcpLabel = activeMcpSource?.label || "";
  const selectedMcpMetaLabel = selectedMcpLabel || t("unselected");
  const selectedSourceLabel =
    selectedSource === "all"
      ? t("allAgentsLabel")
      : sources.find((source) => source.id === selectedSource)?.label || "";
  const currentYear = new Date().getFullYear();

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <div className="brand-title">Ananke</div>
          <div className="brand-sub">{t("tagline")}</div>
          <div className="brand-meta">
            {view === "skills" ? (
              <>
                <div className="meta-chip">
                  {t("countSkills", { count: totalSkills })}
                </div>
              </>
            ) : (
              <>
                <div className="meta-chip">
                  {t("countMcpServers", { count: totalMcpServers })}
                </div>
                <div className="meta-chip">
                  {t("agentMeta", { agent: selectedMcpMetaLabel })}
                </div>
              </>
            )}
          </div>
        </div>
        <div className="controls">
          <div className="view-toggle">
            <button
              className={`toggle-pill ${view === "skills" ? "active" : ""}`}
              onClick={() => setView("skills")}
            >
              {t("viewSkills")}
            </button>
            <button
              className={`toggle-pill ${view === "mcp" ? "active" : ""}`}
              onClick={() => setView("mcp")}
            >
              {t("viewMcp")}
            </button>
          </div>
          <label className="lang-select">
            <span>{t("language")}</span>
            <select
              value={locale}
              onChange={(event) =>
                setLocale(event.target.value as Locale)
              }
            >
              <option value="en">{t("english")}</option>
              <option value="zh">{t("chinese")}</option>
            </select>
          </label>
          <div className="control-buttons">
            {view === "skills" ? (
              <>
                <button
                  className="btn btn-primary"
                  onClick={handleOpenAddSkill}
                >
                  {t("installSkill")}
                </button>
                <button
                  className="btn btn-ghost btn-external"
                  onClick={handleOpenSkillCenter}
                >
                  {t("skillCenter")}
                  <svg
                    className="external-icon"
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                  >
                    <path
                      d="M14 4h6v6M10 14L20 4M20 14v6H4V4h6"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2.2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                </button>
                <button className="btn btn-ghost" onClick={handleRefreshSkills}>
                  {t("refresh")}
                </button>
              </>
            ) : (
              <>
                <button className="btn btn-primary" onClick={handleOpenAddMcp}>
                  {t("addMcpServer")}
                </button>
                <button className="btn btn-ghost" onClick={handleRefreshMcp}>
                  {t("refresh")}
                </button>
              </>
            )}
          </div>
        </div>
      </header>

      {view === "skills" ? (
        <section className="panel-grid">
          <aside className="panel sources">
            <div className="panel-header">
              <h2>{t("aiAgents")}</h2>
              <span className="panel-sub">{t("pickScope")}</span>
            </div>

            <button
              className={`source-card ${
                selectedSource === "all" ? "active" : ""
              }`}
              style={
                {
                  "--accent": sourcePalette.all.accent,
                  "--accent-soft": sourcePalette.all.soft,
                } as CSSProperties
              }
              onClick={() => setSelectedSource("all")}
            >
              <div className="source-title">{t("allAgents")}</div>
              <div className="source-meta">
                <span>{t("countSkills", { count: sourceCount("all") })}</span>
                <span className="status-pill">{t("unifiedView")}</span>
              </div>
            </button>

            <div className="source-list">
              {sources.map((source) => {
                const palette = paletteForSource(source.id);
                return (
                  <button
                    key={source.id}
                    className={`source-card ${
                      selectedSource === source.id ? "active" : ""
                    }`}
                    style={
                      {
                        "--accent": palette.accent,
                        "--accent-soft": palette.soft,
                      } as CSSProperties
                    }
                    onClick={() => setSelectedSource(source.id)}
                  >
                    <div className="source-title">{source.label}</div>
                    <div className="source-meta">
                      <span>
                        {t("countSkills", { count: source.skills.length })}
                      </span>
                      <span className="status-pill">
                        {source.exists ? t("statusReady") : t("statusMissing")}
                      </span>
                    </div>
                    <div className="source-path">{shortenPath(source.root)}</div>
                  </button>
                );
              })}
            </div>
          </aside>

          <section className="panel skills">
            <div className="panel-header">
              <h2>{t("installedSkills")}</h2>
              <span className="panel-sub">
                {selectedSourceLabel}
              </span>
            </div>

            {isLoading ? (
              <div className="empty-state">{t("indexingSkills")}</div>
            ) : error ? (
              <div className="empty-state error">{error}</div>
            ) : visibleSkills.length === 0 ? (
              <div className="empty-state">{t("noSkillsMatch")}</div>
            ) : (
              <div className="skill-list">
                {visibleSkills.map((skill, index) => {
                  const palette = paletteForSource(skill.sourceId);
                  const isActive =
                    selectedSkillKey === `${skill.sourceId}:${skill.id}`;
                  const tags = Object.keys(skill.metadata || {})
                    .filter((key) => {
                      const normalized = key.toLowerCase();
                      return !["name", "description", "license"].includes(
                        normalized,
                      );
                    })
                    .slice(0, 3);

                  return (
                    <button
                      key={`${skill.sourceId}:${skill.id}`}
                      className={`skill-card ${isActive ? "active" : ""}`}
                      style={
                        {
                          "--accent": palette.accent,
                          "--delay": `${index * 0.04}s`,
                        } as CSSProperties
                      }
                      onClick={() => handleSelectSkill(skill)}
                    >
                      <div className="skill-top">
                        <div>
                          <div className="skill-name">{skill.name}</div>
                          <div className="skill-desc">
                            {skill.description || t("noDescription")}
                          </div>
                        </div>
                        <span className="source-pill">{skill.sourceLabel}</span>
                      </div>
                      <div className="skill-meta">
                        <span>{shortenPath(skill.path)}</span>
                        <span>{formatDate(skill.lastModified)}</span>
                      </div>
                      {tags.length > 0 && (
                        <div className="chip-row">
                          {tags.map((tag) => (
                            <span key={tag} className="chip">
                              {tag}
                            </span>
                          ))}
                        </div>
                      )}
                    </button>
                  );
                })}
              </div>
            )}
          </section>

          <aside className="panel detail">
            <div className="panel-header">
              <h2>{t("skillDetails")}</h2>
              <span className="panel-sub">{t("fullInstructions")}</span>
            </div>

            {!selectedSkill ? (
              <div className="empty-state">{t("selectSkillInspect")}</div>
            ) : (
              <div className="detail-content">
                <div className="detail-header">
                  <div>
                    <h3>{selectedSkill.name}</h3>
                    <p>{selectedSkill.description || t("noDescription")}</p>
                  </div>
                  <span className="source-pill">
                    {selectedSkill.sourceLabel}
                  </span>
                </div>

                <div className="detail-grid">
                  <div>
                    <div className="detail-label">{t("labelPath")}</div>
                    <div className="detail-value detail-path">
                      <span>{selectedSkill.path}</span>
                    </div>
                  </div>
                  <div>
                    <div className="detail-label">{t("labelLastModified")}</div>
                    <div className="detail-value">
                      {formatDate(selectedSkill.lastModified)}
                    </div>
                  </div>
                  <div>
                    <div className="detail-label">{t("labelAgentRoot")}</div>
                    <div className="detail-value">
                      {selectedSkill.sourceRoot}
                    </div>
                  </div>
                </div>

                <div>
                  <div className="detail-label">{t("directoryTree")}</div>
                  {skillTreeLoading ? (
                    <div className="empty-state">{t("loadingTree")}</div>
                  ) : skillTreeError ? (
                    <div className="empty-state error">{skillTreeError}</div>
                  ) : skillTree ? (
                    <div className="tree">{renderTreeNode(skillTree)}</div>
                  ) : (
                    <div className="empty-state">{t("noTreeData")}</div>
                  )}
                </div>

                <div>
                  <div className="detail-label">{t("skillContent")}</div>
                  <pre className="detail-body">{selectedSkill.body}</pre>
                </div>

                <div className="detail-actions">
                  {selectedSkill.sourceUrl ? (
                    <button
                      className="btn btn-ghost"
                      onClick={handleSyncSkill}
                      disabled={syncLoading}
                    >
                      {syncLoading ? t("syncing") : t("syncLatest")}
                    </button>
                  ) : null}
                  <button
                    className="btn btn-danger"
                    onClick={handleRequestDeleteSkill}
                  >
                    {t("deleteSkill")}
                  </button>
                </div>
              </div>
            )}
          </aside>
        </section>
      ) : (
        <section className="panel-grid">
          <aside className="panel sources">
            <div className="panel-header">
              <h2>{t("aiAgents")}</h2>
              <span className="panel-sub">{t("mcpScopes")}</span>
            </div>

            <div className="source-list">
              {mcpSources.map((source) => {
                const palette = paletteForSource(source.id);
                return (
                  <button
                    key={source.id}
                    className={`source-card ${
                      selectedMcpSource === source.id ? "active" : ""
                    }`}
                    style={
                      {
                        "--accent": palette.accent,
                        "--accent-soft": palette.soft,
                      } as CSSProperties
                    }
                    onClick={() => setSelectedMcpSource(source.id)}
                  >
                    <div className="source-title">{source.label}</div>
                    <div className="source-meta">
                      <span>
                        {t("countMcpServers", {
                          count: source.servers.length,
                        })}
                      </span>
                      <span className="status-pill">
                        {source.exists ? t("statusReady") : t("statusMissing")}
                      </span>
                    </div>
                    <div className="source-path">{shortenPath(source.path)}</div>
                  </button>
                );
              })}
            </div>
          </aside>

          <section className="panel mcp-list">
            <div className="panel-header">
              <h2>{t("mcpServers")}</h2>
              <span className="panel-sub">
                {activeMcpSource?.label || t("selectAgent")}
              </span>
            </div>

            {activeMcpSource && (
              <div className="mcp-path">{activeMcpSource.path}</div>
            )}

            {mcpLoading ? (
              <div className="empty-state">{t("loadingMcpServers")}</div>
            ) : mcpError ? (
              <div className="empty-state error">{mcpError}</div>
            ) : !activeMcpSource ? (
              <div className="empty-state">{t("selectAgentToViewMcp")}</div>
            ) : activeMcpSource.servers.length === 0 ? (
              <div className="empty-state">{t("noMcpConfigured")}</div>
            ) : (
              <div className="mcp-list-grid">
                {activeMcpSource.servers.map((server, index) => {
                  const config = server.config || {};
                  const command =
                    typeof config.command === "string" ? config.command : null;
                  const url = typeof config.url === "string" ? config.url : null;
                  const args = Array.isArray(config.args)
                    ? config.args.length
                    : 0;

                  return (
                    <button
                      key={server.id}
                      className={`mcp-card ${
                        selectedMcpId === server.id ? "active" : ""
                      }`}
                      style={
                        {
                          "--delay": `${index * 0.05}s`,
                        } as CSSProperties
                      }
                      onClick={() => setSelectedMcpId(server.id)}
                    >
                      <div className="mcp-title">{server.id}</div>
                      <div className="mcp-meta">
                        {url ? <span>{t("urlMeta", { url })}</span> : null}
                        {command ? (
                          <span>{t("cmdMeta", { command })}</span>
                        ) : null}
                        {!url && !command ? (
                          <span>
                            {t("fieldCount", {
                              count: Object.keys(config).length,
                            })}
                          </span>
                        ) : null}
                        {args ? (
                          <span>{t("argsCount", { count: args })}</span>
                        ) : null}
                      </div>
                    </button>
                  );
                })}
              </div>
            )}
          </section>

          <aside className="panel mcp-detail">
            <div className="panel-header">
              <h2>{t("mcpDetails")}</h2>
              <span className="panel-sub">{t("jsonFormat")}</span>
            </div>

            {!selectedMcp ? (
              <div className="empty-state">{t("selectMcpServer")}</div>
            ) : (
              <div className="detail-content">
                <div className="detail-header">
                  <div>
                    <h3>{selectedMcp.id}</h3>
                    <p>{selectedMcpLabel || t("unknownAgent")}</p>
                  </div>
                  <span className="source-pill">
                    {activeMcpSource?.label || t("agentLabel")}
                  </span>
                </div>

                <div>
                  <div className="detail-label">{t("mcpJsonLabel")}</div>
                  <pre className="detail-body">{selectedMcpJson}</pre>
                </div>

                <div className="detail-actions">
                  <button className="btn btn-ghost" onClick={handleEditMcp}>
                    {t("editJson")}
                  </button>
                  <button className="btn btn-danger" onClick={handleRequestDeleteMcp}>
                    {t("deleteServer")}
                  </button>
                </div>
              </div>
            )}
          </aside>
        </section>
      )}

      {showAddSkill && (
        <div className="overlay" role="dialog" aria-modal="true">
          <div className="modal">
            <div className="modal-header">
              <div>
                <h3>{t("installSkillTitle")}</h3>
                <p>{t("installSkillHint")}</p>
              </div>
              <button
                className="btn btn-ghost"
                onClick={() => setShowAddSkill(false)}
              >
                {t("close")}
              </button>
            </div>

            <div className="modal-grid">
              <label>
                <span>{t("targetAgent")}</span>
                <select
                  value={skillForm.sourceId}
                  onChange={(event) =>
                    setSkillForm((current) => ({
                      ...current,
                      sourceId: event.target.value,
                    }))
                  }
                >
                  {sources.map((source) => (
                    <option key={source.id} value={source.id}>
                      {source.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="full">
                <span>{t("githubUrl")}</span>
                <input
                  value={skillForm.url}
                  onChange={(event) =>
                    setSkillForm((current) => ({
                      ...current,
                      url: event.target.value,
                    }))
                  }
                  placeholder={t("githubUrlPlaceholder")}
                />
              </label>
            </div>

            <div className="modal-footer">
              <button className="btn btn-primary" onClick={handleInstallSkill}>
                {t("installSkill")}
              </button>
            </div>
          </div>
        </div>
      )}

      {deleteIntent && (
        <div className="overlay" role="dialog" aria-modal="true">
          <div className="modal">
            <div className="modal-header">
              <div>
                <h3>
                  {deleteIntent.kind === "skill"
                    ? t("confirmDeleteSkillTitle")
                    : t("confirmDeleteServerTitle")}
                </h3>
                <p>{t("irreversible")}</p>
              </div>
              <button
                className="btn btn-ghost"
                onClick={() => setDeleteIntent(null)}
              >
                {t("close")}
              </button>
            </div>

            <div className="modal-grid">
              <label className="full">
                <span>
                  {deleteIntent.kind === "skill"
                    ? t("labelSkill")
                    : t("labelServer")}
                </span>
                <input value={deleteIntent.name} readOnly />
              </label>
            </div>

            <div className="modal-footer">
              <button
                className="btn btn-ghost"
                onClick={() => setDeleteIntent(null)}
              >
                {t("cancel")}
              </button>
              <button className="btn btn-danger" onClick={handleConfirmDelete}>
                {deleteIntent.kind === "skill"
                  ? t("deleteSkill")
                  : t("deleteServer")}
              </button>
            </div>
          </div>
        </div>
      )}

      {showAddMcp && (
        <div className="overlay" role="dialog" aria-modal="true">
          <div className="modal">
            <div className="modal-header">
              <div>
                <h3>{t("registerMcpTitle")}</h3>
                <p>{t("registerMcpHint")}</p>
              </div>
              <button
                className="btn btn-ghost"
                onClick={() => setShowAddMcp(false)}
              >
                {t("close")}
              </button>
            </div>

            <div className="modal-grid">
              <label>
                <span>{t("targetAgent")}</span>
                <select
                  value={mcpForm.sourceId}
                  onChange={(event) =>
                    setMcpForm((current) => ({
                      ...current,
                      sourceId: event.target.value,
                    }))
                  }
                >
                  {mcpSources.map((source) => (
                    <option key={source.id} value={source.id}>
                      {source.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="full">
                <span>{t("mcpJson")}</span>
                <textarea
                  value={mcpForm.json}
                  onChange={(event) =>
                    setMcpForm((current) => ({
                      ...current,
                      json: event.target.value,
                    }))
                  }
                  placeholder={defaultMcpJson}
                />
              </label>
            </div>

            <div className="modal-footer">
              <button className="btn btn-primary" onClick={handleSaveMcp}>
                {t("saveMcp")}
              </button>
            </div>
          </div>
        </div>
      )}

      <footer className="app-footer">
        <span className="footer-text">
          {t("footerCopyright", { year: currentYear })}
        </span>
        <span className="footer-sep">|</span>
        <button className="footer-link" onClick={handleOpenSkillCenter}>
          {t("footerRegistry")}
          <svg className="footer-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M14 4h6v6M10 14L20 4M20 14v6H4V4h6"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </footer>

      {toast && (
        <div className={`toast ${toast.tone}`} role="status">
          {toast.message}
        </div>
      )}
    </div>
  );
}

export default App;
