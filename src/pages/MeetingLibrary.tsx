import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { formatDate, statusLabel, labelTextColor } from "@/lib/utils";
import { useCompactMode } from "@/contexts/CompactModeContext";
import { PageHeader } from "@/components/PageHeader";
import { EmptyState } from "@/components/EmptyState";
import {
  LoadingState,
  randomMeetingsLoadingMessage,
} from "@/components/LoadingState";
import { Card, CardContent } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";
import { Button } from "@/components/ui/button";
import {
  useMeetings,
  deleteMeeting,
  archiveMeeting,
  unarchiveMeeting,
} from "@/hooks/useMeetings";
import { useLabels } from "@/hooks/useLabels";
import { MeetingActionMenuItems } from "@/components/MeetingActionMenuItems";
import { DeleteMeetingDialog } from "@/components/DeleteMeetingDialog";
import { LabelEditor } from "@/components/LabelEditor";
import type { Meeting } from "@/types";
import {
  Search,
  Mic,
  MoreVertical,
  LayoutGrid,
  List,
  Archive,
} from "lucide-react";

function formatDuration(start: string, end: string | null): string {
  if (!end) return "In progress";
  const ms = new Date(end).getTime() - new Date(start).getTime();
  const mins = Math.floor(ms / 60000);
  if (mins < 60) return `${mins}m`;
  const hours = Math.floor(mins / 60);
  const remaining = mins % 60;
  return `${hours}h ${remaining}m`;
}

function statusColor(
  status: string,
): "default" | "secondary" | "outline" | "destructive" {
  switch (status) {
    case "recording":
      return "destructive";
    case "transcribing":
      return "secondary";
    case "summarized":
      return "default";
    default:
      return "outline";
  }
}

const dropdownPrimitives = {
  MenuItem: DropdownMenuItem,
  MenuSeparator: DropdownMenuSeparator,
};

const contextPrimitives = {
  MenuItem: ContextMenuItem,
  MenuSeparator: ContextMenuSeparator,
};

export function MeetingLibrary() {
  const navigate = useNavigate();
  const { isCompact } = useCompactMode();
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [viewMode, setViewMode] = useState<"grid" | "list">(() => {
    return (localStorage.getItem("meetingViewMode") as "grid" | "list") || "grid";
  });
  const [showArchived, setShowArchived] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Meeting | null>(null);
  const [loadingMessage] = useState(randomMeetingsLoadingMessage);
  const [activeLabelIds, setActiveLabelIds] = useState<Set<string>>(new Set());

  // Debounce search input so we don't hit the backend on every keystroke.
  useEffect(() => {
    const t = setTimeout(() => setDebouncedSearch(search), 250);
    return () => clearTimeout(t);
  }, [search]);

  const { meetings, loading, refresh } = useMeetings(
    debouncedSearch || undefined,
    showArchived,
  );
  const { labels, meetingLabelsMap, addMeetingLabel, removeMeetingLabel, createLabel } = useLabels();

  const toggleLabel = useCallback((labelId: string) => {
    setActiveLabelIds((prev) => {
      const next = new Set(prev);
      if (next.has(labelId)) {
        next.delete(labelId);
      } else {
        next.add(labelId);
      }
      return next;
    });
  }, []);

  // Filter meetings by active labels (AND logic: meeting must have ALL selected labels)
  const filteredMeetings = activeLabelIds.size === 0
    ? meetings
    : meetings.filter((meeting) => {
        const meetingLabels = meetingLabelsMap[meeting.id] ?? [];
        const meetingLabelIds = new Set(meetingLabels.map((t) => t.id));
        return Array.from(activeLabelIds).every((labelId) => meetingLabelIds.has(labelId));
      });

  // Drives the empty state copy: "no results" reads very differently from
  // "you haven't recorded anything yet".
  const hasFilters =
    debouncedSearch.trim().length > 0 || activeLabelIds.size > 0;

  const handleViewModeChange = useCallback((mode: "grid" | "list") => {
    setViewMode(mode);
    localStorage.setItem("meetingViewMode", mode);
  }, []);

  const handleArchive = useCallback(
    async (meeting: Meeting) => {
      await archiveMeeting(meeting.id);
      refresh();
    },
    [refresh],
  );

  const handleUnarchive = useCallback(
    async (meeting: Meeting) => {
      await unarchiveMeeting(meeting.id);
      refresh();
    },
    [refresh],
  );

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;
    await deleteMeeting(deleteTarget.id);
    setDeleteTarget(null);
    refresh();
  }, [deleteTarget, refresh]);

  const renderMenuItems = useCallback(
    (
      meeting: Meeting,
      primitives: {
        MenuItem: React.ComponentType<{
          children: React.ReactNode;
          onClick?: () => void;
          className?: string;
        }>;
        MenuSeparator: React.ComponentType<{ className?: string }>;
      },
    ) => (
      <MeetingActionMenuItems
        meeting={meeting}
        onArchive={() => handleArchive(meeting)}
        onUnarchive={() => handleUnarchive(meeting)}
        onDelete={() => setDeleteTarget(meeting)}
        {...primitives}
      />
    ),
    [handleArchive, handleUnarchive],
  );

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PageHeader
        title="Meetings"
        description="Your recorded meetings and transcriptions"
      />

      <div className="flex flex-1 flex-col gap-5 overflow-auto p-6">
      {/* Search and filters */}
      <div className="flex items-center gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search meetings..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <Button
          variant={showArchived ? "secondary" : "outline"}
          size="sm"
          onClick={() => setShowArchived(!showArchived)}
          title={showArchived ? "Hide archived" : "Show archived"}
        >
          <Archive className="h-4 w-4" />
        </Button>
        <div className="flex rounded-md border">
          <Button
            variant={viewMode === "grid" ? "secondary" : "ghost"}
            size="sm"
            className="rounded-r-none border-0"
            onClick={() => handleViewModeChange("grid")}
            title="Grid view"
          >
            <LayoutGrid className="h-4 w-4" />
          </Button>
          <Button
            variant={viewMode === "list" ? "secondary" : "ghost"}
            size="sm"
            className="rounded-l-none border-0"
            onClick={() => handleViewModeChange("list")}
            title="List view"
          >
            <List className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {labels.length > 0 && (
        <div className="flex flex-wrap items-center gap-2">
          {labels.map((label) => {
            const isActive = activeLabelIds.has(label.id);
            return (
              <button
                key={label.id}
                onClick={() => toggleLabel(label.id)}
                className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium transition-colors border ${
                  isActive
                    ? "border-transparent"
                    : "bg-transparent border-border text-foreground hover:bg-accent"
                }`}
                style={
                  isActive
                    ? {
                        backgroundColor: label.color,
                        borderColor: label.color,
                        color: labelTextColor(label.color),
                      }
                    : undefined
                }
              >
                <span
                  className="inline-block h-2 w-2 rounded-full shrink-0"
                  style={{ backgroundColor: label.color }}
                />
                {label.name}
              </button>
            );
          })}
          {activeLabelIds.size > 0 && (
            <button
              onClick={() => setActiveLabelIds(new Set())}
              className="text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              Clear filters
            </button>
          )}
        </div>
      )}

      {/* Meeting content */}
      {loading ? (
        <LoadingState message={loadingMessage} />
      ) : filteredMeetings.length === 0 ? (
        search.toLowerCase() === "noodle" ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-12 text-center">
            <span className="text-4xl">{"\uD83C\uDF5C"}</span>
            <h2 className="text-lg font-medium">You found the secret noodle!</h2>
            <p className="text-sm text-muted-foreground">
              Unfortunately, it's not a meeting.
            </p>
          </div>
        ) : (
          <EmptyState
            icon={Mic}
            title={hasFilters ? "No matching meetings" : "No meetings yet"}
            description={
              hasFilters
                ? "Try a different search or clear your filters."
                : "Hit record and let Nootle do its thing"
            }
          />
        )
      ) : viewMode === "grid" ? (
        <div className={`grid gap-4 ${isCompact ? "grid-cols-1" : "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3"}`}>
          {filteredMeetings.map((meeting, i) => (
            <ContextMenu key={meeting.id}>
              <ContextMenuTrigger asChild>
                <motion.div
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  whileHover={{ y: -2 }}
                  transition={{ duration: 0.2, delay: Math.min(i * 0.04, 0.4) }}
                >
                  <Card
                    className="group cursor-pointer transition-colors hover:bg-accent/30 hover:shadow-md"
                    onClick={() => navigate(`/meeting/${meeting.id}`)}
                  >
                    <CardContent className="space-y-3">
                      <div className="flex items-start justify-between gap-2">
                        <h3 className="font-medium leading-snug line-clamp-2">
                          {meeting.title}
                        </h3>
                        <div className="flex items-center gap-1 shrink-0">
                          <Badge
                            variant={statusColor(meeting.status)}
                          >
                            {statusLabel(meeting.status)}
                          </Badge>
                          <div onClick={(e) => e.stopPropagation()} onPointerDown={(e) => e.stopPropagation()}>
                            <DropdownMenu>
                              <DropdownMenuTrigger asChild>
                                <button
                                  className="opacity-0 group-hover:opacity-100 transition-opacity rounded p-1 hover:bg-accent"
                                >
                                  <MoreVertical className="h-4 w-4 text-muted-foreground" />
                                </button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent align="end">
                                {renderMenuItems(meeting, dropdownPrimitives)}
                              </DropdownMenuContent>
                            </DropdownMenu>
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-3 text-xs text-muted-foreground">
                        <span>{formatDate(meeting.start_time)}</span>
                        <span>{"\u00B7"}</span>
                        <span>
                          {formatDuration(
                            meeting.start_time,
                            meeting.end_time,
                          )}
                        </span>
                      </div>
                      <div onClick={(e) => e.stopPropagation()}>
                        <LabelEditor
                          meetingId={meeting.id}
                          meetingLabels={meetingLabelsMap[meeting.id] ?? []}
                          allLabels={labels}
                          onAddLabel={addMeetingLabel}
                          onRemoveLabel={removeMeetingLabel}
                          onCreateLabel={(name, color) => createLabel(name, color, null)}
                        />
                      </div>
                    </CardContent>
                  </Card>
                </motion.div>
              </ContextMenuTrigger>
              <ContextMenuContent>
                {renderMenuItems(meeting, contextPrimitives)}
              </ContextMenuContent>
            </ContextMenu>
          ))}
        </div>
      ) : (
        <div className="flex flex-col divide-y rounded-md border">
          {filteredMeetings.map((meeting, i) => (
            <ContextMenu key={meeting.id}>
              <ContextMenuTrigger asChild>
                <motion.div
                  initial={{ opacity: 0, x: -8 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.15, delay: Math.min(i * 0.03, 0.3) }}
                  className="group flex items-center gap-4 px-4 py-3 cursor-pointer transition-colors hover:bg-accent/30"
                  onClick={() => navigate(`/meeting/${meeting.id}`)}
                >
                  <h3 className="flex-1 font-medium truncate">
                    {meeting.title}
                  </h3>
                  <div onClick={(e) => e.stopPropagation()} className="shrink-0">
                    <LabelEditor
                      meetingId={meeting.id}
                      meetingLabels={meetingLabelsMap[meeting.id] ?? []}
                      allLabels={labels}
                      onAddLabel={addMeetingLabel}
                      onRemoveLabel={removeMeetingLabel}
                      onCreateLabel={(name, color) => createLabel(name, color, null)}
                    />
                  </div>
                  <span className="text-xs text-muted-foreground whitespace-nowrap">
                    {formatDate(meeting.start_time)}
                  </span>
                  <span className="text-xs text-muted-foreground whitespace-nowrap w-12 text-right">
                    {formatDuration(meeting.start_time, meeting.end_time)}
                  </span>
                  <Badge variant={statusColor(meeting.status)} className="shrink-0">
                    {statusLabel(meeting.status)}
                  </Badge>
                  <div onClick={(e) => e.stopPropagation()} onPointerDown={(e) => e.stopPropagation()}>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <button
                          className="opacity-0 group-hover:opacity-100 transition-opacity rounded p-1 hover:bg-accent"
                        >
                          <MoreVertical className="h-4 w-4 text-muted-foreground" />
                        </button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        {renderMenuItems(meeting, dropdownPrimitives)}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </motion.div>
              </ContextMenuTrigger>
              <ContextMenuContent>
                {renderMenuItems(meeting, contextPrimitives)}
              </ContextMenuContent>
            </ContextMenu>
          ))}
        </div>
      )}

      </div>

      {/* Dialogs */}
      <DeleteMeetingDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        meetingTitle={deleteTarget?.title ?? ""}
        onConfirm={handleDelete}
      />
    </div>
  );
}
