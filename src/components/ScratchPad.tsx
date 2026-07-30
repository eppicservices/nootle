import { useState, useRef, useEffect, useCallback } from "react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useScratchPad } from "@/hooks/useScratchPad";
import { Collapsible } from "@/components/Collapsible";
import { ChevronDown, ChevronRight, X, StickyNote } from "lucide-react";
import { formatMs } from "@/lib/utils";

interface ScratchPadProps {
  meetingId: string | null;
  elapsedMs: number;
}

export function ScratchPad({ meetingId, elapsedMs }: ScratchPadProps) {
  const { notes, addNote, deleteNote } = useScratchPad(meetingId);
  const [isOpen, setIsOpen] = useState(true);
  const [input, setInput] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const handleAdd = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    await addNote(trimmed, elapsedMs);
    setInput("");
  }, [input, elapsedMs, addNote]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        handleAdd();
      }
    },
    [handleAdd],
  );

  // Global keyboard shortcut: Cmd+Shift+N to focus input
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey && e.shiftKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setIsOpen(true);
        setTimeout(() => inputRef.current?.focus(), 50);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <div className="border-t">
      <button
        onClick={() => setIsOpen((isCurrentlyOpen) => !isCurrentlyOpen)}
        className="flex w-full items-center gap-2 px-6 py-2.5 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
      >
        {isOpen ? (
          <ChevronDown className="h-3.5 w-3.5" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5" />
        )}
        <StickyNote className="h-3.5 w-3.5" />
        Quick Notes
        {notes.length > 0 && (
          <Badge variant="warning" size="sm">
            {notes.length}
          </Badge>
        )}
      </button>
      <Collapsible open={isOpen}>
        {isOpen && (
          <div className="px-6 pb-3 space-y-2">
              <div className="flex items-center gap-2">
                <Input
                  ref={inputRef}
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Add a highlight note..."
                  className="h-8 text-xs"
                />
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleAdd}
                  disabled={!input.trim()}
                  className="h-8 px-3 text-xs"
                >
                  Add
                </Button>
              </div>

              {notes.length > 0 && (
                <div className="space-y-1 max-h-[160px] overflow-y-auto">
                  {notes.map((note) => (
                    <motion.div
                      key={note.id}
                      initial={{ opacity: 0, y: -4 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -4 }}
                      className="group flex items-start gap-2 rounded-md border border-highlight/20 bg-highlight/5 px-2.5 py-1.5"
                    >
                      <span className="mt-0.5 shrink-0 font-mono text-xs text-highlight-foreground">
                        {formatMs(note.timestamp_ms)}
                      </span>
                      <span className="text-xs text-foreground flex-1 leading-relaxed">
                        {note.content}
                      </span>
                      <button
                        onClick={() => deleteNote(note.id)}
                        className="text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity mt-0.5 shrink-0"
                        title="Delete note"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </motion.div>
                  ))}
                </div>
              )}
          </div>
        )}
      </Collapsible>
    </div>
  );
}
