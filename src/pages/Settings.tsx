import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Collapsible } from "@/components/Collapsible";
import { PageHeader } from "@/components/PageHeader";
import { CopyButton } from "@/components/CopyButton";
import { LoadingState, LOADING_COPY } from "@/components/LoadingState";
import { INSIGHT_ICONS } from "@/lib/insightIcons";
import { useApiKeys } from "@/hooks/useApiKeys";
import { useLLM } from "@/hooks/useLLM";
import { useModelDownload, MODELS_REQUIRING_AUTH } from "@/hooks/useModelDownload";
import { useTheme } from "@/hooks/useTheme";

import { useInsightTypes } from "@/hooks/useInsightTypes";
import { useAppVersion } from "@/hooks/useAppVersion";
import { AccentColorPicker } from "@/components/AccentColorPicker";
import { EyeOff, Eye, Moon, Sun, Pencil, Trash2, Plus, Link, Unlink } from "lucide-react";
import { formatBytes } from "@/lib/utils";
import { useIntegrations } from "@/hooks/useIntegrations";
import { INTEGRATION_TYPES } from "@/lib/integrations";

const PROVIDERS = ["openai", "anthropic", "google", "groq", "openrouter", "bedrock", "codex"];

const CLI_PROVIDERS = ["claude-agent", "codex-cli"] as const;

const AUTO_DETECTED_PROVIDERS = ["ollama", ...CLI_PROVIDERS] as const;

const PROVIDER_DISPLAY_NAMES: Record<string, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  google: "Google",
  groq: "Groq",
  openrouter: "OpenRouter",
  bedrock: "AWS Bedrock",
  ollama: "Ollama (local)",
  codex: "Codex (API key)",
  "codex-cli": "Codex CLI (ChatGPT subscription)",
  "claude-agent": "Claude CLI (claude -p)",
  linear: "Linear",
  asana: "Asana",
  obsidian: "Obsidian",
};

const PROVIDER_KEY_PLACEHOLDERS: Record<string, string> = {
  bedrock: "us-east-1:ABSK_yourkey…  (region:key)",
  codex: "sk-… (OpenAI API key with Codex access)",
};

const AUTO_DETECTED_HINTS: Record<string, { detected: string; notDetected: string }> = {
  ollama: {
    detected: "Reachable at 127.0.0.1:11434 — no API key needed.",
    notDetected: "Not detected. Start Ollama locally (e.g. `ollama serve`), then restart Nootle.",
  },
  "claude-agent": {
    detected: "Using `claude` on your PATH — no API key needed.",
    notDetected: "Not detected. Install the Claude Code CLI and sign in with your Claude subscription, then restart Nootle.",
  },
  "codex-cli": {
    detected: "Using `codex` on your PATH — no API key needed.",
    notDetected: "Not detected. Install the Codex CLI and sign in with your ChatGPT subscription, then restart Nootle.",
  },
};

function getMcpConfig(exePath: string) {
  return `{
  "mcpServers": {
    "nootle": {
      "command": "${exePath}",
      "args": ["--mcp"]
    }
  }
}`;
}

function getClaudeCommand(exePath: string) {
  return `claude mcp add nootle -- ${exePath} --mcp`;
}


function IntegrationCard({ intType, connectedIntegration, onConnect, onDisconnect }: {
  intType: typeof INTEGRATION_TYPES[number];
  connectedIntegration: { id: string; credentials_json: string } | undefined;
  onConnect: (type: string, name: string, creds: Record<string, string>) => Promise<void>;
  onDisconnect: (id: string) => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(false);
  const [saving, setSaving] = useState(false);
  const [fields, setFields] = useState<Record<string, string>>({});
  const isConnected = !!connectedIntegration;
  const isEmail = intType.type === "email";
  const isObsidian = intType.type === "obsidian";

  const speakerKeys = isObsidian ? (fields._speakerKeys ?? "").split(",").filter(Boolean) : [];

  const handleConnect = async () => {
    if (isEmail) {
      setSaving(true);
      try {
        await onConnect("email", "Email", {});
      } finally {
        setSaving(false);
      }
      return;
    }
    if (!expanded) {
      setExpanded(true);
      return;
    }

    const hasAllRequired = intType.fields.every((f) => fields[f.key]?.trim());
    if (!hasAllRequired) return;

    let creds: Record<string, string> = fields;
    if (isObsidian) {
      const speakerMap: Record<string, string> = {};
      speakerKeys.forEach((key, i) => {
        const value = fields[`_speakerVal_${i}`]?.trim();
        if (key.trim() && value) {
          speakerMap[key.trim()] = value;
        }
      });
      creds = {
        vault_path: fields.vault_path,
        speaker_map: JSON.stringify(speakerMap),
      };
    }

    setSaving(true);
    try {
      await onConnect(intType.type, intType.name, creds);
      setFields({});
      setExpanded(false);
    } finally {
      setSaving(false);
    }
  };

  const handleDisconnect = async () => {
    if (!connectedIntegration) return;
    setSaving(true);
    try {
      await onDisconnect(connectedIntegration.id);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="py-3">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <span className="text-sm font-medium">{intType.name}</span>
          {isConnected ? (
            <Badge variant="success" size="sm">Connected</Badge>
          ) : (
            <Badge variant="secondary" size="sm">Not connected</Badge>
          )}
        </div>
        {isConnected ? (
          <Button variant="outline" size="sm" onClick={handleDisconnect} disabled={saving}>
            <Unlink className="h-3.5 w-3.5 mr-1.5" />
            Disconnect
          </Button>
        ) : (
          <Button variant="outline" size="sm" onClick={handleConnect} disabled={saving}>
            <Link className="h-3.5 w-3.5 mr-1.5" />
            {isEmail ? "Enable" : "Connect"}
          </Button>
        )}
      </div>
      <Collapsible open={expanded && !isConnected && intType.fields.length > 0}>
        <div className="mt-3 space-y-2">
              {intType.fields.map((field) => (
                <div key={field.key} className="flex items-center gap-2">
                  <label className="text-xs text-muted-foreground w-24 shrink-0">{field.label}</label>
                  {field.key === "vault_path" ? (
                    <div className="flex gap-2 flex-1">
                      <Input
                        readOnly
                        placeholder={field.placeholder}
                        value={fields[field.key] ?? ""}
                        className="flex-1"
                      />
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={async () => {
                          const selected = await open({ directory: true, title: "Select Obsidian Vault" });
                          if (selected) {
                            setFields((prev) => ({ ...prev, [field.key]: selected as string }));
                          }
                        }}
                      >
                        Browse
                      </Button>
                    </div>
                  ) : (
                    <Input
                      type="password"
                      placeholder={field.placeholder}
                      value={fields[field.key] ?? ""}
                      onChange={(e) => setFields((prev) => ({ ...prev, [field.key]: e.target.value }))}
                      className="flex-1"
                    />
                  )}
                </div>
              ))}
              {isObsidian && (
                <div className="space-y-2 pt-2">
                  <label className="text-xs font-medium text-muted-foreground">Speaker Mapping</label>
                  <p className="text-xs text-muted-foreground">Map transcript labels to names. Mapped names become [[wikilinks]] in Obsidian.</p>
                  {speakerKeys.map((key, i) => (
                    <div key={i} className="flex items-center gap-2">
                      <Input
                        placeholder="Speaker 1"
                        value={key}
                        onChange={(e) => {
                          const updated = [...speakerKeys];
                          updated[i] = e.target.value;
                          setFields((prev) => ({ ...prev, _speakerKeys: updated.join(",") }));
                        }}
                        className="flex-1"
                      />
                      <span className="text-xs text-muted-foreground">→</span>
                      <Input
                        placeholder="Person Name"
                        value={fields[`_speakerVal_${i}`] ?? ""}
                        onChange={(e) => setFields((prev) => ({ ...prev, [`_speakerVal_${i}`]: e.target.value }))}
                        className="flex-1"
                      />
                      <Button variant="ghost" size="sm" onClick={() => {
                        const vals = speakerKeys.map((_, j) => fields[`_speakerVal_${j}`] ?? "");
                        const nextKeys = speakerKeys.filter((_, j) => j !== i);
                        const nextVals = vals.filter((_, j) => j !== i);
                        const next: Record<string, string> = Object.fromEntries(
                          Object.entries(fields).filter(([k]) => !k.startsWith("_speakerVal_") && k !== "_speakerKeys")
                        );
                        next._speakerKeys = nextKeys.join(",");
                        nextVals.forEach((v, j) => { next[`_speakerVal_${j}`] = v; });
                        setFields(next);
                      }}>
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  ))}
                  <Button variant="outline" size="sm" onClick={() => {
                    setFields((prev) => ({ ...prev, _speakerKeys: [...speakerKeys, ""].join(",") }));
                  }}>
                    <Plus className="h-3.5 w-3.5 mr-1" /> Add Speaker
                  </Button>
                </div>
              )}
              <div className="flex gap-2 justify-end pt-1">
                <Button variant="ghost" size="sm" onClick={() => { setExpanded(false); setFields({}); }}>
                  Cancel
                </Button>
                <Button
                  size="sm"
                  onClick={handleConnect}
                  disabled={saving || !intType.fields.every((f) => fields[f.key]?.trim())}
                >
                  {saving ? "Saving..." : "Save"}
                </Button>
              </div>
        </div>
      </Collapsible>
    </div>
  );
}

function IntegrationsManager() {
  const { integrations, loading, createIntegration, deleteIntegration } = useIntegrations();

  const handleConnect = async (type: string, name: string, creds: Record<string, string>) => {
    await createIntegration(type, name, JSON.stringify(creds));
  };

  const handleDisconnect = async (id: string) => {
    await deleteIntegration(id);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Integrations</CardTitle>
        <CardDescription>
          Connect services to use in post-meeting workflows. These are separate from the API keys above, which are used for LLM providers.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <LoadingState message={LOADING_COPY.integrations} layout="inline" />
        ) : (
          <div className="divide-y">
            {INTEGRATION_TYPES.map((intType) => (
              <IntegrationCard
                key={intType.type}
                intType={intType}
                connectedIntegration={integrations.find((i) => i.integration_type === intType.type)}
                onConnect={handleConnect}
                onDisconnect={handleDisconnect}
              />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}


function ApiKeyRow({ provider, isStored, onSave, onDelete }: {
  provider: string;
  isStored: boolean;
  onSave: (key: string) => Promise<void>;
  onDelete: () => Promise<void>;
}) {
  const [editing, setEditing] = useState(false);
  const [keyValue, setKeyValue] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!keyValue.trim()) return;
    setSaving(true);
    try {
      await onSave(keyValue);
      setKeyValue("");
      setEditing(false);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex items-center gap-3 py-3">
      <div className="flex items-center gap-2 w-32 shrink-0">
        <span className="text-sm font-medium">{PROVIDER_DISPLAY_NAMES[provider] ?? provider}</span>
        {isStored && <Badge variant="success" size="sm">Saved</Badge>}
      </div>

      {editing ? (
        <div className="flex flex-1 items-center gap-2">
          <Input
            type={showKey ? "text" : "password"}
            placeholder={PROVIDER_KEY_PLACEHOLDERS[provider] ?? `Enter ${provider} API key`}
            value={keyValue}
            onChange={(e) => setKeyValue(e.target.value)}
            className="flex-1"
          />
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => setShowKey(!showKey)}
            title={showKey ? "Hide" : "Show"}
          >
            {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </Button>
          <Button size="sm" onClick={handleSave} disabled={saving || !keyValue.trim()}>
            {saving ? "..." : "Save"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setEditing(false);
              setKeyValue("");
            }}
          >
            Cancel
          </Button>
        </div>
      ) : (
        <div className="flex flex-1 items-center gap-2">
          {isStored ? (
            <>
              <span className="flex-1 text-sm text-muted-foreground font-mono">
                {"*".repeat(24)}
              </span>
              <Button variant="outline" size="sm" onClick={() => setEditing(true)}>
                Update
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="text-destructive hover:text-destructive"
                onClick={onDelete}
              >
                Delete
              </Button>
            </>
          ) : (
            <>
              <span className="flex-1 text-sm text-muted-foreground">
                Not configured
              </span>
              <Button variant="outline" size="sm" onClick={() => setEditing(true)}>
                Add Key
              </Button>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function AutoDetectedProviderRow({ provider, detected }: { provider: string; detected: boolean }) {
  const hint = AUTO_DETECTED_HINTS[provider];
  return (
    <div className="flex items-center gap-3 py-3">
      <div className="flex items-center gap-2 w-48 shrink-0">
        <span className="text-sm font-medium">{PROVIDER_DISPLAY_NAMES[provider] ?? provider}</span>
        {detected && <Badge variant="success" size="sm">Detected</Badge>}
      </div>
      <div className="flex-1 text-sm text-muted-foreground">
        {detected ? hint?.detected : hint?.notDetected}
      </div>
    </div>
  );
}

function InsightTypesManager() {
  const { types, createInsightType, updateInsightType, deleteInsightType } = useInsightTypes();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editDesc, setEditDesc] = useState("");
  const [editPrompt, setEditPrompt] = useState("");
  const [editIcon, setEditIcon] = useState("");
  const [editHasAction, setEditHasAction] = useState(false);
  const [adding, setAdding] = useState(false);
  const [newName, setNewName] = useState("");
  const [newSlug, setNewSlug] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [newPrompt, setNewPrompt] = useState("");
  const [newIcon, setNewIcon] = useState("lightbulb");
  const [newHasAction, setNewHasAction] = useState(false);

  const startEditing = (t: typeof types[0]) => {
    setEditingId(t.id);
    setEditName(t.name);
    setEditDesc(t.description ?? "");
    setEditPrompt(t.extraction_prompt);
    setEditIcon(t.icon);
    setEditHasAction(t.has_action_fields);
  };

  const handleSaveEdit = async (id: string) => {
    await updateInsightType(
      id,
      editName,
      editDesc || null,
      editPrompt,
      editIcon,
      editHasAction,
    );
    setEditingId(null);
  };

  const handleCreate = async () => {
    if (!newName.trim() || !newSlug.trim() || !newPrompt.trim()) return;
    await createInsightType(newName, newSlug, newDesc || null, newPrompt, newIcon, newHasAction);
    setAdding(false);
    setNewName("");
    setNewSlug("");
    setNewDesc("");
    setNewPrompt("");
    setNewIcon("lightbulb");
    setNewHasAction(false);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Insight Types</CardTitle>
        <CardDescription>
          Configure what types of insights to extract from meetings and customize the extraction prompts
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {types.map((t) => (
          <div key={t.id} className="border rounded-lg p-4 space-y-3">
            {editingId === t.id ? (
              <>
                <div className="flex items-center gap-2">
                  <Input
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    className="h-8 flex-1"
                    placeholder="Name"
                  />
                  <Select
                    size="sm"
                    containerClassName="w-40"
                    value={editIcon}
                    onChange={(e) => setEditIcon(e.target.value)}
                    aria-label="Icon"
                  >
                    {INSIGHT_ICONS.map((entry) => (
                      <option key={entry.value} value={entry.value}>
                        {entry.label}
                      </option>
                    ))}
                  </Select>
                </div>
                <Input
                  value={editDesc}
                  onChange={(e) => setEditDesc(e.target.value)}
                  className="h-8"
                  placeholder="Description"
                />
                <Textarea
                  value={editPrompt}
                  onChange={(e) => setEditPrompt(e.target.value)}
                  className="min-h-20"
                  placeholder="Extraction prompt"
                />
                <label className="flex w-fit cursor-pointer items-center gap-2 text-sm">
                  <Checkbox
                    checked={editHasAction}
                    onCheckedChange={(checked) => setEditHasAction(checked === true)}
                  />
                  Has action fields (assignee, due date)
                </label>
                <div className="flex gap-2">
                  <Button size="sm" onClick={() => handleSaveEdit(t.id)}>Save</Button>
                  <Button variant="ghost" size="sm" onClick={() => setEditingId(null)}>Cancel</Button>
                </div>
              </>
            ) : (
              <>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-semibold">{t.name}</span>
                    <Badge variant="secondary" size="sm">{t.slug}</Badge>
                    {t.is_builtin && <Badge variant="outline" size="sm">Built-in</Badge>}
                    {t.has_action_fields && <Badge variant="outline" size="sm">Action fields</Badge>}
                  </div>
                  <div className="flex gap-1">
                    <Button variant="ghost" size="icon-sm" onClick={() => startEditing(t)}>
                      <Pencil className="h-3.5 w-3.5" />
                    </Button>
                    {!t.is_builtin && (
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        className="text-destructive hover:text-destructive"
                        onClick={() => deleteInsightType(t.id)}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    )}
                  </div>
                </div>
                {t.description && (
                  <p className="text-xs text-muted-foreground">{t.description}</p>
                )}
                <details className="text-xs">
                  <summary className="cursor-pointer text-muted-foreground hover:text-foreground">
                    Extraction prompt
                  </summary>
                  <pre className="mt-1 whitespace-pre-wrap rounded bg-muted p-2 text-xs">
                    {t.extraction_prompt}
                  </pre>
                </details>
              </>
            )}
          </div>
        ))}

        {adding ? (
          <div className="border rounded-lg p-4 space-y-3 border-dashed">
            <div className="flex gap-2">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="h-8 flex-1"
                placeholder="Name (e.g. Risk)"
              />
              <Input
                value={newSlug}
                onChange={(e) => setNewSlug(e.target.value)}
                className="h-8 w-40"
                placeholder="Slug (e.g. risk)"
              />
              <Select
                size="sm"
                containerClassName="w-40"
                value={newIcon}
                onChange={(e) => setNewIcon(e.target.value)}
                aria-label="Icon"
              >
                {INSIGHT_ICONS.map((entry) => (
                  <option key={entry.value} value={entry.value}>
                    {entry.label}
                  </option>
                ))}
              </Select>
            </div>
            <Input
              value={newDesc}
              onChange={(e) => setNewDesc(e.target.value)}
              className="h-8"
              placeholder="Description (optional)"
            />
            <Textarea
              value={newPrompt}
              onChange={(e) => setNewPrompt(e.target.value)}
              className="min-h-20"
              placeholder="Extraction prompt — tell the LLM how to identify this type"
            />
            <label className="flex w-fit cursor-pointer items-center gap-2 text-sm">
              <Checkbox
                checked={newHasAction}
                onCheckedChange={(checked) => setNewHasAction(checked === true)}
              />
              Has action fields (assignee, due date)
            </label>
            <div className="flex gap-2">
              <Button size="sm" onClick={handleCreate} disabled={!newName.trim() || !newSlug.trim() || !newPrompt.trim()}>
                Create
              </Button>
              <Button variant="ghost" size="sm" onClick={() => setAdding(false)}>Cancel</Button>
            </div>
          </div>
        ) : (
          <Button variant="outline" size="sm" onClick={() => setAdding(true)}>
            <Plus className="h-4 w-4 mr-1" /> Add Custom Type
          </Button>
        )}
      </CardContent>
    </Card>
  );
}


export function SettingsPage() {
  const { storedProviders, storeKey, deleteKey } = useApiKeys();
  const { providers: llmProviders } = useLLM();
  const { theme, toggleTheme } = useTheme();
  const version = useAppVersion();
  const [exePath, setExePath] = useState("/path/to/nootle");
  const [denoiseEnabled, setDenoiseEnabled] = useState(true);
  const [detectionEnabled, setDetectionEnabled] = useState(true);

  useEffect(() => {
    invoke<string>("get_exe_path").then(setExePath).catch(() => {});
    invoke<string | null>("get_app_setting", { key: "denoise_enabled" })
      .then((val) => setDenoiseEnabled(val !== "false"))
      .catch(() => {});
    invoke<string | null>("get_app_setting", { key: "detection_enabled" })
      .then((val) => setDetectionEnabled(val !== "false"))
      .catch(() => {});
  }, []);

  const toggleDenoise = async (enabled: boolean) => {
    setDenoiseEnabled(enabled);
    await invoke("set_app_setting", {
      key: "denoise_enabled",
      value: String(enabled),
    });
  };

  const toggleDetection = async (enabled: boolean) => {
    setDetectionEnabled(enabled);
    await invoke("set_app_setting", {
      key: "detection_enabled",
      value: String(enabled),
    });
  };

  const mcpConfig = getMcpConfig(exePath);
  const claudeCommand = getClaudeCommand(exePath);

  // Exclude auto-detected providers — they're rendered in their own card below
  // since they don't take API keys.
  const autoDetectedSet = new Set<string>(AUTO_DETECTED_PROVIDERS);
  const allProviders = Array.from(
    new Set([...PROVIDERS, ...llmProviders]),
  ).filter((p) => !autoDetectedSet.has(p));

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PageHeader
        title="Settings"
        description="Configure API keys and application settings"
      />

      <Tabs defaultValue="general" className="flex flex-1 flex-col overflow-hidden">
        <div className="shrink-0 border-b px-6 py-4">
          <TabsList className="h-10">
            <TabsTrigger value="general">General</TabsTrigger>
            <TabsTrigger value="api-keys">API Keys</TabsTrigger>
            <TabsTrigger value="integrations">Integrations</TabsTrigger>
            <TabsTrigger value="models">Models</TabsTrigger>
            <TabsTrigger value="insight-types">Insight Types</TabsTrigger>
            <TabsTrigger value="about">About / MCP</TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="general" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <Card>
              <CardHeader>
                <CardTitle>Appearance</CardTitle>
                <CardDescription>Choose your preferred color scheme</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Theme</p>
                    <p className="text-sm text-muted-foreground">
                      {theme === "light" ? "Light mode" : "Dark mode"}
                    </p>
                  </div>
                  <Button variant="outline" size="sm" onClick={toggleTheme}>
                    {theme === "light" ? <><Moon className="h-4 w-4" /> Dark</> : <><Sun className="h-4 w-4" /> Light</>}
                  </Button>
                </div>
                <AccentColorPicker />
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Recording</CardTitle>
                <CardDescription>Configure audio recording behavior</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Noise cancellation</p>
                    <p className="text-sm text-muted-foreground">
                      Clean up audio before transcription for better accuracy
                    </p>
                  </div>
                  <Switch
                    checked={denoiseEnabled}
                    onCheckedChange={toggleDenoise}
                    aria-label="Noise cancellation"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Auto-detect meetings</p>
                    <p className="text-sm text-muted-foreground">
                      Get notified when a meeting is detected so you can start recording
                    </p>
                  </div>
                  <Switch
                    checked={detectionEnabled}
                    onCheckedChange={toggleDetection}
                    aria-label="Auto-detect meetings"
                  />
                </div>
              </CardContent>
            </Card>
            <PermissionsCard />
          </div>
        </TabsContent>

        <TabsContent value="api-keys" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <Card>
              <CardHeader>
                <CardTitle>API Keys</CardTitle>
                <CardDescription>
                  Configure API keys for LLM providers. Local Ollama and
                  subscription CLIs are auto-detected — see the next section.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="divide-y">
                  {allProviders.map((provider) => (
                    <ApiKeyRow
                      key={provider}
                      provider={provider}
                      isStored={storedProviders.includes(provider)}
                      onSave={(key) => storeKey(provider, key)}
                      onDelete={() => deleteKey(provider)}
                    />
                  ))}
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Auto-detected providers</CardTitle>
                <CardDescription>
                  Local Ollama and subscription CLIs don't need an API key —
                  Nootle picks them up on startup. Each row shows whether it
                  was detected.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="divide-y">
                  {AUTO_DETECTED_PROVIDERS.map((provider) => (
                    <AutoDetectedProviderRow
                      key={provider}
                      provider={provider}
                      detected={llmProviders.includes(provider)}
                    />
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="integrations" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <IntegrationsManager />
          </div>
        </TabsContent>

        <TabsContent value="models" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <ModelManagementCard />
          </div>
        </TabsContent>


        <TabsContent value="insight-types" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <InsightTypesManager />
          </div>
        </TabsContent>

        <TabsContent value="about" className="mt-0 flex-1 overflow-auto">
          <div className="flex flex-col gap-8 p-6 max-w-3xl">
            <Card>
              <CardHeader>
                <CardTitle>About</CardTitle>
                <CardDescription>
                  Nootle v{version} — Your meetings, transcribed and summarized with a twist
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <h3 className="text-sm font-medium mb-2">MCP Server Configuration</h3>
                  <p className="text-xs text-muted-foreground mb-3">
                    Add this to your MCP client configuration to use Nootle as an MCP
                    server:
                  </p>
                  <div className="relative">
                    <pre className="overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">
                      {mcpConfig}
                    </pre>
                    <CopyButton
                      variant="button"
                      text={mcpConfig}
                      className="absolute top-2 right-2"
                    />
                  </div>
                  <h3 className="mt-4 mb-2 text-sm font-medium">Claude Code</h3>
                  <p className="mb-3 text-xs text-muted-foreground">
                    Or install directly with Claude Code:
                  </p>
                  <div className="relative">
                    <pre className="overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">
                      {claudeCommand}
                    </pre>
                    <CopyButton
                      variant="button"
                      text={claudeCommand}
                      className="absolute top-2 right-2"
                    />
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

function PermissionsCard() {
  const [permissions, setPermissions] = useState<{
    microphone: string;
    screen_recording: boolean;
    calendar: string;
  } | null>(null);
  const [requesting, setRequesting] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const status = await invoke<{
        microphone: string;
        screen_recording: boolean;
        calendar: string;
      }>("check_permissions");
      setPermissions(status);
    } catch {}
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleRequest = useCallback(async (type: "microphone" | "screen_recording" | "calendar") => {
    setRequesting(type);
    try {
      if (type === "microphone") {
        await invoke("request_microphone_permission");
      } else if (type === "screen_recording") {
        await invoke("request_screen_recording_permission");
      } else {
        await invoke("request_calendar_permission");
      }
      await refresh();
    } finally {
      setRequesting(null);
    }
  }, [refresh]);

  const statusBadge = (granted: boolean) => (
    <Badge variant={granted ? "success" : "secondary"} size="sm">
      {granted ? "Granted" : "Not granted"}
    </Badge>
  );

  const rows: { label: string; key: "microphone" | "screen_recording" | "calendar"; granted: boolean; description: string }[] = permissions ? [
    { label: "Microphone", key: "microphone", granted: permissions.microphone === "granted", description: "Required for recording audio" },
    { label: "Screen Recording", key: "screen_recording", granted: permissions.screen_recording, description: "Required for system audio capture" },
    { label: "Calendar", key: "calendar", granted: permissions.calendar === "granted", description: "Auto-detect meetings from your calendar" },
  ] : [];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Permissions</CardTitle>
        <CardDescription>
          Manage macOS permissions for recording and calendar access
        </CardDescription>
      </CardHeader>
      <CardContent>
        {!permissions ? (
          <LoadingState message={LOADING_COPY.permissions} layout="inline" />
        ) : (
          <div className="divide-y">
            {rows.map((row) => (
              <div key={row.key} className="flex items-center gap-3 py-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{row.label}</span>
                    {statusBadge(row.granted)}
                  </div>
                  <p className="text-xs text-muted-foreground mt-0.5">{row.description}</p>
                </div>
                {!row.granted && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleRequest(row.key)}
                    disabled={requesting !== null}
                  >
                    {requesting === row.key ? "..." : "Grant"}
                  </Button>
                )}
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function ModelManagementCard() {
  const {
    registry,
    diskStatus,
    progress,
    downloadModel,
    cancelDownload,
    deleteModel,
  } = useModelDownload();

  const [selectedVariants, setSelectedVariants] = useState<
    Record<string, string>
  >({});
  const [deleting, setDeleting] = useState<string | null>(null);

  const isDownloading =
    progress !== null &&
    typeof progress.state === "string" &&
    (progress.state === "downloading" || progress.state === "verifying");

  const getSelectedVariant = (modelId: string): string => {
    if (selectedVariants[modelId]) return selectedVariants[modelId];
    const model = registry.find((m) => m.id === modelId);
    if (!model) return "default";
    if (model.variants.length === 1) return model.variants[0].id;
    const int8 = model.variants.find((v) => v.id === "int8");
    return int8 ? "int8" : model.variants[0].id;
  };

  const handleDelete = async (modelId: string) => {
    setDeleting(modelId);
    try {
      await deleteModel(modelId);
    } finally {
      setDeleting(null);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>AI Models</CardTitle>
        <CardDescription>
          Manage local AI models for transcription and speaker identification.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="divide-y">
          {diskStatus
            .filter((model) => !MODELS_REQUIRING_AUTH.has(model.model_id))
            .map((model) => {
            const regModel = registry.find((r) => r.id === model.model_id);
            const isThisModelDownloading =
              isDownloading && progress?.model_id === model.model_id;
            const errorMessage =
              progress?.model_id === model.model_id &&
              typeof progress.state !== "string" &&
              "error" in progress.state
                ? progress.state.error.message
                : null;

            return (
              <div key={model.model_id} className="py-4 first:pt-0 last:pb-0">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-sm font-medium">
                        {model.name}
                      </span>
                      <Badge
                        variant={model.downloaded ? "success" : "secondary"}
                        size="sm"
                      >
                        {model.downloaded ? "Downloaded" : "Not downloaded"}
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {model.description}
                    </p>

                    {model.downloaded && model.size_on_disk > 0 && (
                      <p className="text-xs text-muted-foreground mt-1">
                        Size on disk: {formatBytes(model.size_on_disk)}
                      </p>
                    )}

                    {/* Variant picker for not-downloaded models with multiple variants */}
                    {!model.downloaded &&
                      regModel &&
                      regModel.variants.length > 1 && (
                        <div className="flex gap-3 mt-2">
                          {regModel.variants.map((variant) => (
                            <label
                              key={variant.id}
                              className="flex items-center gap-2 cursor-pointer"
                            >
                              <input
                                type="radio"
                                name={`settings-variant-${model.model_id}`}
                                checked={
                                  getSelectedVariant(model.model_id) ===
                                  variant.id
                                }
                                onChange={() =>
                                  setSelectedVariants((prev) => ({
                                    ...prev,
                                    [model.model_id]: variant.id,
                                  }))
                                }
                                className="accent-primary"
                              />
                              <span className="text-xs text-foreground">
                                {variant.label}
                              </span>
                              <span className="text-xs text-muted-foreground">
                                ({formatBytes(variant.total_size_bytes)})
                              </span>
                            </label>
                          ))}
                        </div>
                      )}

                    {/* Progress bar for this model */}
                    {isThisModelDownloading && progress && (
                      <div className="mt-2">
                        <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
                          <span>
                            {typeof progress.state === "string" &&
                            progress.state === "verifying"
                              ? "Verifying..."
                              : `Downloading ${progress.current_file}`}
                          </span>
                          <span>
                            {Math.round(progress.overall_percent * 100)}%
                          </span>
                        </div>
                        <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                          <div
                            className="h-full rounded-full bg-primary transition-[width] duration-300"
                            style={{
                              width: `${Math.round(progress.overall_percent * 100)}%`,
                            }}
                          />
                        </div>
                      </div>
                    )}

                    {errorMessage && (
                      <p className="mt-2 text-xs text-destructive">
                        Download failed: {errorMessage}
                      </p>
                    )}
                  </div>

                  {/* Action buttons */}
                  <div className="flex items-center gap-2 shrink-0 pt-0.5">
                    {isThisModelDownloading ? (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={cancelDownload}
                      >
                        Cancel
                      </Button>
                    ) : model.downloaded ? (
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-destructive hover:text-destructive"
                        onClick={() => handleDelete(model.model_id)}
                        disabled={deleting === model.model_id}
                      >
                        {deleting === model.model_id ? "Deleting..." : "Delete"}
                      </Button>
                    ) : (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                          downloadModel(
                            model.model_id,
                            getSelectedVariant(model.model_id)
                          )
                        }
                        disabled={isDownloading}
                      >
                        Download
                      </Button>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
