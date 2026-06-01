import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useAllInsights } from "@/hooks/useInsights";
import { useApiKeys } from "@/hooks/useApiKeys";
import { useLLM } from "@/hooks/useLLM";
import { useLinearTeams, useLinearProjects, useLinearSettings } from "@/hooks/useLinear";
import type { InsightWithActionItem, InsightType, LinearTeam } from "@/types";
import { Check, Lightbulb, ListChecks, Star, Search, AlertTriangle, Ticket } from "lucide-react";
import { formatDate } from "@/lib/utils";

const ICON_MAP: Record<string, React.ComponentType<{ className?: string }>> = {
  lightbulb: Lightbulb,
  "list-checks": ListChecks,
  star: Star,
  "alert-triangle": AlertTriangle,
};

function getIcon(iconName: string): React.ComponentType<{ className?: string }> {
  return ICON_MAP[iconName] ?? Lightbulb;
}

function ActionItemTicketButton({
  item,
  teams,
  onTicketCreated,
}: {
  item: InsightWithActionItem;
  teams: LinearTeam[];
  onTicketCreated: () => void;
}) {
  const { storedProviders } = useApiKeys();
  const { models, providers } = useLLM();
  const { defaultTeamId, defaultProjectId } = useLinearSettings();
  const { projects } = useLinearProjects(defaultTeamId);
  const [open, setOpen] = useState(false);
  const [teamId, setTeamId] = useState(defaultTeamId ?? "");
  const [projectId, setProjectId] = useState(defaultProjectId ?? "");
  const [provider, setProvider] = useState(providers[0] ?? "");
  const [model, setModel] = useState("");
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!storedProviders.includes("linear")) return null;

  if (item.linear_ticket_id) {
    return (
      <Badge variant="secondary" className="text-[10px] shrink-0">
        <Ticket className="h-3 w-3 mr-1" />
        {item.linear_ticket_id}
      </Badge>
    );
  }

  const filteredModels = models.filter((m) => m.provider === provider);

  const handleCreate = async () => {
    if (!item.action_item_id || !teamId || !provider || !model) return;
    setCreating(true);
    setError(null);
    try {
      await invoke("create_ticket_from_action_item", {
        actionItemId: item.action_item_id,
        teamId,
        projectId: projectId || null,
        provider,
        model,
      });
      setOpen(false);
      onTicketCreated();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  };

  if (!open) {
    return (
      <button
        onClick={(e) => {
          e.stopPropagation();
          setOpen(true);
        }}
        className="shrink-0 p-1 rounded text-muted-foreground hover:text-primary transition-colors"
        title="Create Linear ticket"
      >
        <Ticket className="h-3.5 w-3.5" />
      </button>
    );
  }

  return (
    <div
      className="shrink-0 border rounded-md p-2 space-y-2 bg-background"
      onClick={(e) => e.stopPropagation()}
    >
      <div className="flex gap-1.5">
        <select
          value={teamId}
          onChange={(e) => setTeamId(e.target.value)}
          className="h-7 flex-1 rounded border bg-transparent px-1 text-[11px]"
        >
          <option value="">Team</option>
          {teams.map((t) => (
            <option key={t.id} value={t.id}>{t.name}</option>
          ))}
        </select>
        <select
          value={projectId}
          onChange={(e) => setProjectId(e.target.value)}
          className="h-7 flex-1 rounded border bg-transparent px-1 text-[11px]"
        >
          <option value="">Project</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      </div>
      <div className="flex gap-1.5">
        <select
          value={provider}
          onChange={(e) => {
            setProvider(e.target.value);
            setModel("");
          }}
          className="h-7 flex-1 rounded border bg-transparent px-1 text-[11px]"
        >
          <option value="">Provider</option>
          {providers.map((p) => (
            <option key={p} value={p}>{p}</option>
          ))}
        </select>
        <select
          value={model}
          onChange={(e) => setModel(e.target.value)}
          className="h-7 flex-1 rounded border bg-transparent px-1 text-[11px]"
        >
          <option value="">Model</option>
          {filteredModels.map((m) => (
            <option key={m.id} value={m.id}>{m.name}</option>
          ))}
        </select>
      </div>
      <div className="flex gap-1.5">
        <Button
          size="xs"
          className="flex-1"
          onClick={handleCreate}
          disabled={creating || !teamId || !provider || !model}
        >
          {creating ? "Creating..." : "Create Ticket"}
        </Button>
        <Button variant="ghost" size="xs" onClick={() => setOpen(false)}>
          Cancel
        </Button>
      </div>
      {error && <p className="text-[10px] text-destructive">{error}</p>}
    </div>
  );
}

function DashboardActionItem({
  item,
  teams,
  onToggle,
  onNavigate,
  onTicketCreated,
}: {
  item: InsightWithActionItem;
  teams: LinearTeam[];
  onToggle: (actionItemId: string, currentStatus: string) => void;
  onNavigate: (meetingId: string) => void;
  onTicketCreated: () => void;
}) {
  const isDone = item.status === "done";

  return (
    <motion.div
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      className="flex items-start gap-3 rounded-md border p-3 transition-colors hover:bg-accent/30"
    >
      <button
        onClick={(e) => {
          e.stopPropagation();
          if (item.action_item_id) onToggle(item.action_item_id, item.status ?? "open");
        }}
        className={`mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors ${
          isDone
            ? "border-primary bg-primary text-primary-foreground"
            : "border-muted-foreground hover:border-primary"
        }`}
      >
        {isDone && <Check className="h-3 w-3" />}
      </button>
      <div
        className="min-w-0 flex-1 cursor-pointer"
        onClick={() => onNavigate(item.meeting_id)}
      >
        <p className={`text-sm leading-relaxed ${isDone ? "line-through text-muted-foreground" : "text-foreground"}`}>
          {item.content}
        </p>
        <div className="mt-1 flex flex-wrap items-center gap-2">
          <Badge variant={isDone ? "secondary" : "outline"} className="text-[10px]">
            {isDone ? "Done" : "Open"}
          </Badge>
          {item.assignee && (
            <span className="text-[10px] text-muted-foreground">{item.assignee}</span>
          )}
          {item.due_date && (
            <span className="text-[10px] text-muted-foreground">{item.due_date}</span>
          )}
          {item.meeting_title && (
            <span className="text-[10px] text-muted-foreground truncate max-w-[200px]">
              {item.meeting_title}
            </span>
          )}
          {item.meeting_start_time && (
            <span className="text-[10px] text-muted-foreground">
              {formatDate(item.meeting_start_time)}
            </span>
          )}
        </div>
      </div>
      <ActionItemTicketButton item={item} teams={teams} onTicketCreated={onTicketCreated} />
    </motion.div>
  );
}

function InsightItem({
  item,
  icon: Icon,
  onNavigate,
}: {
  item: InsightWithActionItem;
  icon: React.ComponentType<{ className?: string }>;
  onNavigate: (meetingId: string) => void;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      className="flex items-start gap-3 rounded-md border p-3 cursor-pointer transition-colors hover:bg-accent/30"
      onClick={() => onNavigate(item.meeting_id)}
    >
      <Icon className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
      <div className="min-w-0 flex-1">
        <p className="text-sm leading-relaxed">{item.content}</p>
        <div className="mt-1 flex flex-wrap items-center gap-2">
          {item.meeting_title && (
            <span className="text-[10px] text-muted-foreground truncate max-w-[200px]">
              {item.meeting_title}
            </span>
          )}
          {item.meeting_start_time && (
            <span className="text-[10px] text-muted-foreground">
              {formatDate(item.meeting_start_time)}
            </span>
          )}
        </div>
      </div>
    </motion.div>
  );
}

function SectionHeader({
  icon: Icon,
  title,
  count,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  count: number;
}) {
  return (
    <div className="flex items-center gap-2 pb-2">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <h2 className="text-lg font-semibold">{title}</h2>
      <Badge variant="secondary">{count}</Badge>
    </div>
  );
}

function TypeSection({
  insightType,
  items,
  teams,
  toggleActionItem,
  onNavigate,
  onTicketCreated,
}: {
  insightType: InsightType;
  items: InsightWithActionItem[];
  teams: LinearTeam[];
  toggleActionItem: (actionItemId: string, currentStatus: string) => void;
  onNavigate: (meetingId: string) => void;
  onTicketCreated: () => void;
}) {
  const Icon = getIcon(insightType.icon);

  if (insightType.has_action_fields) {
    const openItems = items.filter((i) => i.status !== "done");
    const doneItems = items.filter((i) => i.status === "done");
    return (
      <section>
        <SectionHeader icon={Icon} title={insightType.name + "s"} count={items.length} />
        <div className="space-y-2">
          {openItems.length === 0 && doneItems.length === 0 && (
            <p className="text-sm text-muted-foreground italic">No {insightType.name.toLowerCase()}s found</p>
          )}
          {openItems.map((item) => (
            <DashboardActionItem
              key={item.id}
              item={item}
              teams={teams}
              onToggle={toggleActionItem}
              onNavigate={onNavigate}
              onTicketCreated={onTicketCreated}
            />
          ))}
          {doneItems.map((item) => (
            <DashboardActionItem
              key={item.id}
              item={item}
              teams={teams}
              onToggle={toggleActionItem}
              onNavigate={onNavigate}
              onTicketCreated={onTicketCreated}
            />
          ))}
        </div>
      </section>
    );
  }

  return (
    <section>
      <SectionHeader icon={Icon} title={insightType.name + "s"} count={items.length} />
      <div className="space-y-2">
        {items.length === 0 && (
          <p className="text-sm text-muted-foreground italic">No {insightType.name.toLowerCase()}s found</p>
        )}
        {items.map((item) => (
          <InsightItem
            key={item.id}
            item={item}
            icon={Icon}
            onNavigate={onNavigate}
          />
        ))}
      </div>
    </section>
  );
}

export function InsightsDashboard() {
  const navigate = useNavigate();
  const [typeFilter, setTypeFilter] = useState<string | undefined>(undefined);
  const [statusFilter, setStatusFilter] = useState<string | undefined>(undefined);
  const [search, setSearch] = useState("");
  const [searchDebounced, setSearchDebounced] = useState<string | undefined>(undefined);
  const { teams, fetchTeams } = useLinearTeams();

  useEffect(() => {
    fetchTeams();
  }, [fetchTeams]);

  const { insightTypes, groupedByType, loading, toggleActionItem, refresh } = useAllInsights(
    typeFilter,
    statusFilter,
    searchDebounced,
  );

  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const handleSearchChange = (value: string) => {
    setSearch(value);
    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setSearchDebounced(value || undefined);
    }, 300);
  };

  useEffect(() => {
    return () => clearTimeout(debounceRef.current);
  }, []);

  const handleNavigate = (meetingId: string) => {
    navigate(`/meeting/${meetingId}`);
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex flex-1 flex-col"
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-4 border-b px-6 py-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Insights</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Action items, decisions, and key moments extracted from your meetings
          </p>
        </div>
        <div className="flex items-center gap-3 shrink-0 pt-1">
          <select
            value={typeFilter ?? ""}
            onChange={(e) => setTypeFilter(e.target.value || undefined)}
            className="h-8 rounded-md border bg-transparent px-2 text-sm"
          >
            <option value="">All types</option>
            {insightTypes.map((t) => (
              <option key={t.slug} value={t.slug}>{t.name}s</option>
            ))}
          </select>
          <select
            value={statusFilter ?? ""}
            onChange={(e) => setStatusFilter(e.target.value || undefined)}
            className="h-8 rounded-md border bg-transparent px-2 text-sm"
          >
            <option value="">All statuses</option>
            <option value="open">Open</option>
            <option value="done">Done</option>
          </select>
          <div className="relative">
            <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search insights..."
              value={search}
              onChange={(e) => handleSearchChange(e.target.value)}
              className="h-8 w-56 pl-8 text-sm"
            />
          </div>
        </div>
      </div>

      {/* Content */}
      {loading ? (
        <div className="flex flex-1 items-center justify-center">
          <p className="text-sm text-muted-foreground">Fishing out the good bits...</p>
        </div>
      ) : (
        <ScrollArea className="flex-1">
          <div className="space-y-8 p-6">
            {insightTypes
              .filter((t) => typeFilter === undefined || typeFilter === t.slug)
              .map((t) => (
                <TypeSection
                  key={t.slug}
                  insightType={t}
                  items={groupedByType[t.slug] ?? []}
                  teams={teams}
                  toggleActionItem={toggleActionItem}
                  onNavigate={handleNavigate}
                  onTicketCreated={refresh}
                />
              ))}
          </div>
        </ScrollArea>
      )}
    </motion.div>
  );
}
