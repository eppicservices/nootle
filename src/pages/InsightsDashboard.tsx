import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { PageHeader } from "@/components/PageHeader";
import { LoadingState, LOADING_COPY } from "@/components/LoadingState";
import { useAllInsights } from "@/hooks/useInsights";
import { useApiKeys } from "@/hooks/useApiKeys";
import { useLLM } from "@/hooks/useLLM";
import { useLinearTeams, useLinearProjects, useLinearSettings } from "@/hooks/useLinear";
import type { InsightWithActionItem, InsightType, LinearTeam } from "@/types";
import type { LucideIcon } from "lucide-react";
import { Search, Ticket } from "lucide-react";
import { ActionItemCheckbox } from "@/components/ActionItemCheckbox";
import { formatDate } from "@/lib/utils";
import { insightIcon } from "@/lib/insightIcons";

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
      <Badge variant="secondary" size="sm" className="shrink-0">
        <Ticket />
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
        <Select
          size="xs"
          containerClassName="flex-1"
          value={teamId}
          onChange={(e) => setTeamId(e.target.value)}
          aria-label="Linear team"
        >
          <option value="">Team</option>
          {teams.map((t) => (
            <option key={t.id} value={t.id}>{t.name}</option>
          ))}
        </Select>
        <Select
          size="xs"
          containerClassName="flex-1"
          value={projectId}
          onChange={(e) => setProjectId(e.target.value)}
          aria-label="Linear project"
        >
          <option value="">Project</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </Select>
      </div>
      <div className="flex gap-1.5">
        <Select
          size="xs"
          containerClassName="flex-1"
          value={provider}
          onChange={(e) => {
            setProvider(e.target.value);
            setModel("");
          }}
          aria-label="LLM provider"
        >
          <option value="">Provider</option>
          {providers.map((p) => (
            <option key={p} value={p}>{p}</option>
          ))}
        </Select>
        <Select
          size="xs"
          containerClassName="flex-1"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          aria-label="LLM model"
        >
          <option value="">Model</option>
          {filteredModels.map((m) => (
            <option key={m.id} value={m.id}>{m.name}</option>
          ))}
        </Select>
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
      {error && <p className="text-xs text-destructive">{error}</p>}
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
      <ActionItemCheckbox
        done={isDone}
        label={item.content}
        onToggle={() => {
          if (item.action_item_id) onToggle(item.action_item_id, item.status ?? "open");
        }}
      />
      <div
        className="min-w-0 flex-1 cursor-pointer"
        onClick={() => onNavigate(item.meeting_id)}
      >
        <p className={`text-sm leading-relaxed ${isDone ? "line-through text-muted-foreground" : "text-foreground"}`}>
          {item.content}
        </p>
        <div className="mt-1.5 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <Badge variant={isDone ? "secondary" : "outline"} size="sm">
            {isDone ? "Done" : "Open"}
          </Badge>
          {item.assignee && <span>{item.assignee}</span>}
          {item.due_date && <span>{item.due_date}</span>}
          {item.meeting_title && (
            <span className="max-w-[200px] truncate">{item.meeting_title}</span>
          )}
          {item.meeting_start_time && (
            <span>{formatDate(item.meeting_start_time)}</span>
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
  icon: LucideIcon;
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
        <div className="mt-1.5 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          {item.meeting_title && (
            <span className="max-w-[200px] truncate">{item.meeting_title}</span>
          )}
          {item.meeting_start_time && (
            <span>{formatDate(item.meeting_start_time)}</span>
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
  icon: LucideIcon;
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
  const Icon = insightIcon(insightType.icon);

  if (insightType.has_action_fields) {
    // Open items first, done ones sink to the bottom.
    const ordered = [
      ...items.filter((i) => i.status !== "done"),
      ...items.filter((i) => i.status === "done"),
    ];
    return (
      <section>
        <SectionHeader icon={Icon} title={insightType.name + "s"} count={items.length} />
        <div className="space-y-2">
          {ordered.length === 0 && (
            <p className="text-sm text-muted-foreground italic">No {insightType.name.toLowerCase()}s found</p>
          )}
          {ordered.map((item) => (
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
      <PageHeader
        title="Insights"
        description="Action items, decisions, and key moments extracted from your meetings"
        actions={
          <>
            <Select
              size="sm"
              value={typeFilter ?? ""}
              onChange={(e) => setTypeFilter(e.target.value || undefined)}
              aria-label="Filter by insight type"
            >
              <option value="">All types</option>
              {insightTypes.map((t) => (
                <option key={t.slug} value={t.slug}>{t.name}s</option>
              ))}
            </Select>
            <Select
              size="sm"
              value={statusFilter ?? ""}
              onChange={(e) => setStatusFilter(e.target.value || undefined)}
              aria-label="Filter by status"
            >
              <option value="">All statuses</option>
              <option value="open">Open</option>
              <option value="done">Done</option>
            </Select>
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search insights..."
                value={search}
                onChange={(e) => handleSearchChange(e.target.value)}
                className="h-8 w-56 pl-8"
              />
            </div>
          </>
        }
      />

      {/* Content */}
      {loading ? (
        <LoadingState message={LOADING_COPY.insights} />
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
