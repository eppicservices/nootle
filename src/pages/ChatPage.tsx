import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { EmptyState } from "@/components/EmptyState";
import { Markdown } from "@/components/Markdown";
import { ThinkingDots } from "@/components/ThinkingDots";
import { ResizeHandle } from "@/components/ResizeHandle";
import { useChatConversations, useChatMessages } from "@/hooks/useChatHistory";
import { useLabels } from "@/hooks/useLabels";
import { useGlobalLLMSelection } from "@/contexts/LLMSelectionContext";
import { useCompactMode } from "@/contexts/CompactModeContext";
import type { ChatSource, GlobalChatResponse } from "@/types";
import { Plus, Trash2, MessageSquare, Send, PanelLeftOpen, PanelLeftClose } from "lucide-react";
import { SourceCitation } from "@/components/SourceCitation";

export function ChatPage() {
  const navigate = useNavigate();
  const { conversations, refresh: refreshConvos, createConversation, deleteConversation, updateTitle } =
    useChatConversations();
  const [activeId, setActiveId] = useState<string | null>(null);
  const { messages: dbMessages, refresh: refreshMessages } = useChatMessages(activeId);
  const { labels } = useLabels();
  const { selectedProvider, selectedModel } = useGlobalLLMSelection();
  const { isCompact } = useCompactMode();

  const [editingTitleId, setEditingTitleId] = useState<string | null>(null);
  const [titleDraft, setTitleDraft] = useState("");
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [selectedLabel, setSelectedLabel] = useState("");
  const [dateFromValue, setDateFromValue] = useState("");
  const [dateToValue, setDateToValue] = useState("");
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Resize state for conversation list
  const [sidebarWidth, setSidebarWidth] = useState(256);

  const showSidebar = !isCompact || sidebarOpen;

  useEffect(() => {
    if (conversations.length > 0 && !activeId) {
      setActiveId(conversations[0].id);
    }
  }, [conversations, activeId]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [dbMessages]);

  const getDateFrom = () => dateFromValue ? new Date(dateFromValue).toISOString() : null;
  const getDateTo = () => dateToValue ? new Date(dateToValue + "T23:59:59").toISOString() : null;

  const handleNewConversation = async () => {
    const conv = await createConversation();
    setActiveId(conv.id);
  };

  const handleDelete = async (id: string) => {
    await deleteConversation(id);
    if (activeId === id) {
      setActiveId(null);
    }
  };

  const handleTitleSave = async (id: string) => {
    const conv = conversations.find((c) => c.id === id);
    if (!titleDraft.trim() || titleDraft.trim() === conv?.title) {
      setEditingTitleId(null);
      return;
    }
    await updateTitle(id, titleDraft.trim());
    setEditingTitleId(null);
  };

  const handleSend = async () => {
    if (!input.trim() || !selectedProvider || !selectedModel || !activeId) return;
    const msg = input;
    setInput("");
    setLoading(true);
    try {
      await invoke<GlobalChatResponse>("send_chat_message", {
        conversationId: activeId,
        message: msg,
        provider: selectedProvider,
        model: selectedModel,
        labelIds: selectedLabel ? [selectedLabel] : [],
        dateFrom: getDateFrom(),
        dateTo: getDateTo(),
      });
      await refreshMessages();
      await refreshConvos();
    } catch (err) {
      console.error("Chat error:", err);
    } finally {
      setLoading(false);
    }
  };

  // Parse sources from a message's sources_json field
  const parseSources = (sourcesJson: string | null): ChatSource[] => {
    if (!sourcesJson) return [];
    try {
      return JSON.parse(sourcesJson);
    } catch {
      return [];
    }
  };

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Left panel - conversation list */}
      {showSidebar && (
      <div
        style={{ width: isCompact ? "100%" : sidebarWidth }}
        className="relative flex shrink-0 flex-col border-r bg-background"
      >
        <div className="flex items-center justify-between px-4 h-12 border-b">
          <h2 className="text-sm font-semibold">Conversations</h2>
          <div className="flex items-center gap-1">
            <Button variant="ghost" size="icon-sm" onClick={handleNewConversation}>
              <Plus className="h-4 w-4" />
            </Button>
            {isCompact && (
              <Button variant="ghost" size="icon-sm" onClick={() => setSidebarOpen(false)} title="Hide conversations">
                <PanelLeftClose className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="p-2 space-y-1">
            {conversations.length === 0 && (
              <p className="text-xs text-muted-foreground p-2">Start a conversation using the + button above</p>
            )}
            {conversations.map((conv) => (
              <div
                key={conv.id}
                className={`group flex items-center gap-2 rounded-md px-3 py-2 cursor-pointer transition-colors min-w-0 ${
                  activeId === conv.id
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent/50"
                }`}
                onClick={() => {
                  setActiveId(conv.id);
                  if (isCompact) setSidebarOpen(false);
                }}
              >
                <MessageSquare className="h-3.5 w-3.5 shrink-0" />
                {editingTitleId === conv.id ? (
                  <input
                    autoFocus
                    value={titleDraft}
                    onChange={(e) => setTitleDraft(e.target.value)}
                    onBlur={() => handleTitleSave(conv.id)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleTitleSave(conv.id);
                      if (e.key === "Escape") setEditingTitleId(null);
                    }}
                    onClick={(e) => e.stopPropagation()}
                    className="flex-1 min-w-0 bg-transparent text-sm border-none outline-none"
                  />
                ) : (
                  <span
                    className="flex-1 min-w-0 truncate text-sm"
                    onDoubleClick={(e) => {
                      e.stopPropagation();
                      setTitleDraft(conv.title);
                      setEditingTitleId(conv.id);
                    }}
                  >
                    {conv.title}
                  </span>
                )}
                <button
                  className="opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(conv.id);
                  }}
                >
                  <Trash2 className="h-3 w-3 text-muted-foreground hover:text-destructive" />
                </button>
              </div>
            ))}
          </div>
        </ScrollArea>
        {!isCompact && (
          <ResizeHandle
            width={sidebarWidth}
            onWidthChange={setSidebarWidth}
            min={180}
            max={480}
            side="right"
            label="Resize conversation list"
          />
        )}
      </div>
      )}

      {/* Right panel - chat */}
      {(!isCompact || !sidebarOpen) && (
      <div className="flex flex-1 flex-col">
        {/* Filters bar */}
        <div className="flex items-center gap-3 px-4 h-12 border-b">
          {isCompact && (
            <Button variant="ghost" size="icon-sm" onClick={() => setSidebarOpen(true)} title="Show conversations">
              <PanelLeftOpen className="h-4 w-4" />
            </Button>
          )}
          <Select
            size="xs"
            value={selectedLabel}
            onChange={(e) => setSelectedLabel(e.target.value)}
            aria-label="Filter by label"
          >
            <option value="">All labels</option>
            {labels.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </Select>
          <Input
            type="date"
            value={dateFromValue}
            onChange={(e) => setDateFromValue(e.target.value)}
            className="h-7 w-auto px-2 text-xs"
            aria-label="From date"
          />
          <span className="text-xs text-muted-foreground">to</span>
          <Input
            type="date"
            value={dateToValue}
            onChange={(e) => setDateToValue(e.target.value)}
            className="h-7 w-auto px-2 text-xs"
            aria-label="To date"
          />
        </div>

        {/* Messages area */}
        {!activeId ? (
          <EmptyState
            icon={MessageSquare}
            description="Nootle remembers everything from your meetings — just ask"
            action={
              <Button size="sm" onClick={handleNewConversation}>
                <Plus /> New Conversation
              </Button>
            }
          />
        ) : (
          <>
            <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-4">
              {dbMessages.length === 0 && !loading && (
                <div className="flex items-center justify-center h-full">
                  <p className="text-sm text-muted-foreground">
                    What happened in that meeting? Go ahead, ask anything
                  </p>
                </div>
              )}
              {dbMessages.map((msg) => {
                const sources = msg.role === "assistant" ? parseSources(msg.sources_json) : [];
                return (
                  <motion.div
                    key={msg.id}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
                  >
                    <div
                      className={`max-w-[80%] rounded-lg px-4 py-2.5 ${
                        msg.role === "user"
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted"
                      }`}
                    >
                      {msg.role === "assistant" ? (
                          <Markdown content={msg.content} />
                      ) : (
                        <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                      )}
                      {sources.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-1">
                          {sources.map((s, i) => (
                            <SourceCitation
                              key={i}
                              source={s}
                              onClick={() => navigate(`/meeting/${s.meeting_id}`)}
                            />
                          ))}
                        </div>
                      )}
                    </div>
                  </motion.div>
                );
              })}
              {loading && (
                <div className="flex justify-start">
                  <div className="bg-muted rounded-lg px-4 py-2.5">
                    <ThinkingDots />
                  </div>
                </div>
              )}
            </div>

            {/* Input bar */}
            <div className="border-t p-4">
              {!selectedProvider || !selectedModel ? (
                <p className="text-sm text-muted-foreground text-center py-2">
                  Select a model in the sidebar to start chatting. You can add API keys in Settings.
                </p>
              ) : (
                <div className="flex gap-2">
                  <Input
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Ask about your meetings..."
                    className="flex-1"
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleSend();
                      }
                    }}
                    disabled={loading}
                  />
                  <Button
                    onClick={handleSend}
                    disabled={loading || !input.trim()}
                  >
                    <Send className="h-4 w-4" />
                  </Button>
                </div>
              )}
            </div>
          </>
        )}
      </div>
      )}
    </div>
  );
}
