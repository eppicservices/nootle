import { useState, useRef, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { MotionButton } from "@/components/MotionButton";
import { ThinkingDots } from "@/components/ThinkingDots";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Markdown } from "@/components/Markdown";
import { useGlobalChat } from "@/hooks/useGlobalChat";
import { useLabels } from "@/hooks/useLabels";
import { useGlobalLLMSelection } from "@/contexts/LLMSelectionContext";
import {
  X,
  MessageSquare,
  GripHorizontal,
} from "lucide-react";
import { SourceCitation } from "@/components/SourceCitation";
import { useCompactMode } from "@/contexts/CompactModeContext";

const DATE_PRESETS = [
  { label: "Last 7 days", days: 7 },
  { label: "Last 30 days", days: 30 },
  { label: "Last 90 days", days: 90 },
  { label: "All time", days: null },
] as const;

export function GlobalChatPanel() {
  const [open, setOpen] = useState(false);
  const {
    messages,
    loading,
    error,
    sendMessage,
    clearMessages,
    setFilters,
    embeddingStatus,
    embedAllMeetings,
  } = useGlobalChat();
  const { labels } = useLabels();
  const { selectedProvider, selectedModel } = useGlobalLLMSelection();
  const navigate = useNavigate();
  const location = useLocation();
  const onChatPage = location.pathname === "/chat";
  // Meeting detail page has an audio player pinned to the bottom; lift the
  // chat bubble so it doesn't sit on top of the timestamp readout.
  const onMeetingPage = location.pathname.startsWith("/meeting/");

  const [input, setInput] = useState("");
  const [selectedLabel, setSelectedLabel] = useState(""); // "" = all
  const [selectedDatePreset, setSelectedDatePreset] = useState(3); // "All time"
  const [embedding, setEmbedding] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Dragging state – use right/bottom offsets so framer-motion's transform doesn't conflict
  const [offset, setOffset] = useState({ right: 24, bottom: 24 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, right: 0, bottom: 0 });

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || !selectedProvider || !selectedModel) return;
    const msg = input;
    setInput("");
    await sendMessage(msg, selectedProvider, selectedModel);
  };

  const currentDateFrom = () => {
    const preset = DATE_PRESETS[selectedDatePreset];
    return preset.days
      ? new Date(Date.now() - preset.days * 86400000).toISOString()
      : null;
  };

  const handleDatePresetChange = (idx: number) => {
    setSelectedDatePreset(idx);
    const preset = DATE_PRESETS[idx];
    const dateFrom = preset.days
      ? new Date(Date.now() - preset.days * 86400000).toISOString()
      : null;
    setFilters(selectedLabel ? [selectedLabel] : [], dateFrom, null);
  };

  const handleLabelChange = (labelId: string) => {
    setSelectedLabel(labelId);
    setFilters(labelId ? [labelId] : [], currentDateFrom(), null);
  };

  const handleEmbedAll = async () => {
    setEmbedding(true);
    await embedAllMeetings();
    setEmbedding(false);
  };

  const handleDragStart = (e: React.MouseEvent) => {
    setIsDragging(true);
    dragStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      right: offset.right,
      bottom: offset.bottom,
    };
  };

  useEffect(() => {
    if (!isDragging) return;
    const PANEL_WIDTH = 400;
    const PANEL_HEIGHT = 600;
    const MARGIN = 8;
    const handleMove = (e: MouseEvent) => {
      const rawRight =
        dragStartRef.current.right - (e.clientX - dragStartRef.current.x);
      const rawBottom =
        dragStartRef.current.bottom - (e.clientY - dragStartRef.current.y);
      // Clamp so the panel stays on-screen.
      const maxRight = Math.max(MARGIN, window.innerWidth - PANEL_WIDTH - MARGIN);
      const maxBottom = Math.max(MARGIN, window.innerHeight - PANEL_HEIGHT - MARGIN);
      setOffset({
        right: Math.min(Math.max(rawRight, MARGIN), maxRight),
        bottom: Math.min(Math.max(rawBottom, MARGIN), maxBottom),
      });
    };
    const handleUp = () => setIsDragging(false);
    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
    return () => {
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
  }, [isDragging]);

  const modelNotReady =
    embeddingStatus && !embeddingStatus.model_available;
  const needsBackfill =
    embeddingStatus &&
    embeddingStatus.model_available &&
    embeddingStatus.embedded < embeddingStatus.total;

  const { isCompact } = useCompactMode();

  if (onChatPage || isCompact) return null;

  return (
    <>
      {/* Floating Action Button */}
      <AnimatePresence initial={false}>
        {!open && (
          <motion.button
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            exit={{ scale: 0 }}
            onClick={() => setOpen(true)}
            className={`fixed right-6 z-40 flex h-12 w-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg hover:shadow-xl transition-shadow ${onMeetingPage ? "bottom-24" : "bottom-6"}`}
            title="Ask across meetings"
          >
            <MessageSquare className="h-5 w-5" />
          </motion.button>
        )}
      </AnimatePresence>

      {/* Chat Panel */}
      <AnimatePresence>
        {open && (
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            style={{
              right: offset.right,
              bottom: offset.bottom,
            }}
            className="fixed z-50 flex w-[400px] h-[600px] flex-col rounded-xl border bg-background shadow-2xl"
          >
            {/* Header */}
            <div
              onMouseDown={handleDragStart}
              className="flex items-center justify-between px-4 py-3 border-b cursor-grab active:cursor-grabbing select-none"
            >
              <div className="flex items-center gap-2">
                <GripHorizontal className="h-4 w-4 text-muted-foreground" />
                <h3 className="font-semibold text-sm">
                  Ask across meetings
                </h3>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                onMouseDown={(e) => e.stopPropagation()}
                onClick={() => setOpen(false)}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>

            {/* Filters */}
            <div className="flex flex-col gap-2 px-4 py-2 border-b">
              <div className="flex items-center gap-2">
                <select
                  value={selectedLabel}
                  onChange={(e) => handleLabelChange(e.target.value)}
                  aria-label="Filter by label"
                  className="h-7 flex-1 rounded-md border bg-transparent px-2 text-xs"
                >
                  <option value="">All Labels</option>
                  {labels.map((label) => (
                    <option key={label.id} value={label.id}>
                      {label.name}
                    </option>
                  ))}
                </select>
                <select
                  value={selectedDatePreset}
                  onChange={(e) =>
                    handleDatePresetChange(Number(e.target.value))
                  }
                  aria-label="Filter by date range"
                  className="h-7 rounded-md border bg-transparent px-2 text-xs"
                >
                  {DATE_PRESETS.map((preset, i) => (
                    <option key={i} value={i}>
                      {preset.label}
                    </option>
                  ))}
                </select>
              </div>

              {embeddingStatus && (
                <div className="flex items-center justify-between">
                  <p className="text-xs text-muted-foreground">
                    {embeddingStatus.embedded} of {embeddingStatus.total}{" "}
                    meetings indexed
                  </p>
                  {!modelNotReady && needsBackfill && (
                    <Button
                      variant="outline"
                      size="xs"
                      onClick={handleEmbedAll}
                      disabled={embedding}
                    >
                      {embedding ? "Indexing..." : `Index ${embeddingStatus.total - embeddingStatus.embedded} meetings`}
                    </Button>
                  )}
                </div>
              )}
            </div>

            {/* Messages */}
            <ScrollArea className="flex-1">
              <div
                ref={scrollRef}
                className="flex flex-col gap-3 p-4"
              >
                {modelNotReady && (
                  <div className="rounded-lg bg-muted p-4 text-center text-sm text-muted-foreground">
                    <p className="mb-2">
                      Download the search model in Settings to
                      search across meetings.
                    </p>
                  </div>
                )}

                {messages.length === 0 &&
                  !modelNotReady && (
                    <p className="text-center text-sm text-muted-foreground py-8">
                      Nootle has total recall — ask away
                    </p>
                  )}

                {messages.map((msg, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.15 }}
                    className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
                  >
                    <div
                      className={`max-w-[85%] rounded-lg px-3 py-2 text-sm ${
                        msg.role === "user"
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted text-foreground"
                      }`}
                    >
                      {msg.role === "assistant" ? (
                        <Markdown content={msg.content} />
                      ) : (
                        msg.content
                      )}
                      {msg.sources && msg.sources.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-1">
                          {msg.sources.map((source, j) => (
                            <SourceCitation
                              key={j}
                              source={source}
                              onClick={() => {
                                setOpen(false);
                                navigate(
                                  `/meeting/${source.meeting_id}`
                                );
                              }}
                            />
                          ))}
                        </div>
                      )}
                    </div>
                  </motion.div>
                ))}

                {loading && (
                  <div className="flex justify-start">
                    <div className="rounded-lg bg-muted px-3 py-2 text-sm text-muted-foreground">
                      <ThinkingDots />
                    </div>
                  </div>
                )}

                {error && (
                  <p className="text-xs text-destructive text-center">
                    {error}
                  </p>
                )}
              </div>
            </ScrollArea>

            <Separator />

            {/* Input */}
            <div className="flex items-center gap-2 p-3">
              <Input
                placeholder="Ask about your meetings..."
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    handleSend();
                  }
                }}
                disabled={loading || !!modelNotReady}
                className="flex-1"
              />
              <MotionButton
                size="sm"
                onClick={handleSend}
                disabled={loading || !input.trim() || !!modelNotReady}
              >
                Ask
              </MotionButton>
            </div>

            {/* Clear button */}
            <div className="px-3 pb-3">
              <Button
                variant="ghost"
                size="xs"
                className="w-full text-muted-foreground"
                onClick={clearMessages}
              >
                Clear conversation
              </Button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
