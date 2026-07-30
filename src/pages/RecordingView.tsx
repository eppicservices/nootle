import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { useNavigate, Link } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { MotionButton } from "@/components/MotionButton";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useRecording } from "@/hooks/useRecording";
import { useTemplates } from "@/hooks/useTemplates";
import type { TranscriptSegment } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ScratchPad } from "@/components/ScratchPad";
import { Collapsible } from "@/components/Collapsible";
import { useCompactMode } from "@/contexts/CompactModeContext";
import { CompactRecordingIndicator } from "@/components/CompactRecordingIndicator";
import { Square, ArrowLeft, ChevronDown, ChevronRight, FileText } from "lucide-react";

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function WaveformBar({ index }: { index: number }) {
  const { maxHeight, animDuration } = useMemo(() => ({
    maxHeight: 10 + Math.random() * 6,
    animDuration: 0.6 + Math.random() * 0.4,
  }), []);

  return (
    <motion.div
      className="w-[2px] rounded-full bg-primary"
      initial={{ height: 0, opacity: 0 }}
      animate={{
        height: [3, maxHeight, 3],
        opacity: 1,
      }}
      transition={{
        height: {
          duration: animDuration,
          repeat: Infinity,
          repeatType: "reverse",
          delay: 0.3 + index * 0.05,
          ease: "easeInOut",
        },
        opacity: {
          duration: 0.2,
          delay: index * 0.03,
        },
      }}
    />
  );
}

interface TranscriptionStatus {
  available: boolean;
  reason?: string;
}

export function RecordingView() {
  const navigate = useNavigate();
  const { isRecording, currentMeeting, elapsed, error, startRecording, stopRecording } =
    useRecording();
  const { templates } = useTemplates();
  const { isCompact } = useCompactMode();
  const [title, setTitle] = useState("Untitled Recording");
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [hasStarted, setHasStarted] = useState(false);
  const [segments, setSegments] = useState<TranscriptSegment[]>([]);
  const [notes, setNotes] = useState("");
  const [stopping, setStopping] = useState(false);
  const [transcriptOpen, setTranscriptOpen] = useState(false);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("");
  const [transcriptionStatus, setTranscriptionStatus] =
    useState<TranscriptionStatus | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const notesRef = useRef<HTMLTextAreaElement>(null);

  // Listen for transcript updates
  useEffect(() => {
    const unlisten = listen<TranscriptSegment[]>(
      "transcript-update",
      (event) => {
        setSegments(event.payload);
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for transcription status
  useEffect(() => {
    const unlisten = listen<TranscriptionStatus>(
      "transcription-status",
      (event) => {
        setTranscriptionStatus(event.payload);
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const latestTitleRef = useRef(title);
  latestTitleRef.current = title;
  const latestTemplateRef = useRef(selectedTemplateId);
  latestTemplateRef.current = selectedTemplateId;

  // Start recording on mount — after event listeners are registered above
  // so we don't miss the transcription-status event from the backend
  useEffect(() => {
    if (!hasStarted) {
      setHasStarted(true);
      startRecording(
        latestTitleRef.current,
        undefined,
        latestTemplateRef.current || undefined,
      ).catch(() => {
        // Error is captured in useRecording's error state
      });
    }
  }, [hasStarted, startRecording]);

  // Recording starts the moment this view mounts, so a template picked
  // afterwards has to be pushed to the meeting that's already in flight.
  const handleTemplateChange = useCallback(
    async (templateId: string) => {
      setSelectedTemplateId(templateId);
      if (!currentMeeting) return;
      try {
        await invoke("update_meeting_template", {
          id: currentMeeting.id,
          templateId: templateId || null,
        });
      } catch (err) {
        console.error("Failed to update meeting template:", err);
      }
    },
    [currentMeeting],
  );

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [segments]);

  const handleStop = useCallback(async () => {
    setStopping(true);
    await new Promise((r) => setTimeout(r, 400));
    try {
      const meeting = await stopRecording();
      if (notes.trim()) {
        await invoke("save_meeting_notes", { id: meeting.id, rawNotes: notes });
      }
      navigate(`/meeting/${meeting.id}`);
    } catch {
      navigate("/");
    }
  }, [stopRecording, navigate, notes]);

  // Show error state if recording failed to start
  if (error && !isRecording) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-6 p-8">
        <div className="rounded-lg border border-destructive/50 bg-destructive/5 p-6 text-center max-w-md">
          <h2 className="text-lg font-semibold text-foreground mb-2">
            Recording Failed
          </h2>
          <p className="text-sm text-destructive mb-4">{error}</p>
          <Button variant="outline" onClick={() => navigate("/")}>
            <ArrowLeft className="h-4 w-4 mr-2" />
            Go Back
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col">
      {/* Header bar: recording indicator, title, timer, waveform, stop */}
      <div className="flex items-center gap-4 border-b px-6 py-3">
        <div className="flex items-center gap-2">
          <motion.div
            className="h-2.5 w-2.5 rounded-full bg-destructive"
            animate={{ opacity: [1, 0.3, 1] }}
            transition={{ duration: 1.5, repeat: Infinity }}
          />
          <span className="text-xs font-medium text-muted-foreground">REC</span>
        </div>

        {isCompact ? (
          <span className="text-sm font-semibold truncate max-w-[120px]">
            {title}
          </span>
        ) : (
          <>
            {isEditingTitle ? (
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                onBlur={() => setIsEditingTitle(false)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") setIsEditingTitle(false);
                }}
                className="text-sm font-semibold border-none bg-transparent h-auto py-0 max-w-xs"
                autoFocus
              />
            ) : (
              <button
                className="text-sm font-semibold hover:text-muted-foreground transition-colors truncate max-w-xs"
                onClick={() => setIsEditingTitle(true)}
              >
                {title}
              </button>
            )}

            <div className="flex items-center gap-1.5">
              <FileText className="h-3.5 w-3.5 text-muted-foreground" />
              <Select
                size="xs"
                value={selectedTemplateId}
                onChange={(e) => handleTemplateChange(e.target.value)}
                aria-label="Summary template"
                title="Summarize with this template when the recording ends"
              >
                <option value="">No template</option>
                {templates.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </Select>
            </div>
          </>
        )}

        <span className="font-mono text-sm tabular-nums text-muted-foreground">
          {formatTime(elapsed)}
        </span>

        {isRecording && !isCompact && (
          <div className="flex items-center gap-[2px] h-4">
            {Array.from({ length: 12 }).map((_, i) => (
              <WaveformBar key={i} index={i} />
            ))}
          </div>
        )}

        <div className="ml-auto relative inline-flex">
          <MotionButton
            size="sm"
            variant="destructive"
            onClick={handleStop}
            disabled={stopping}
          >
            <Square className="h-3.5 w-3.5" /> Stop
          </MotionButton>
          <AnimatePresence>
            {stopping && (
              <motion.div
                className="absolute inset-0 rounded-md border-2 border-destructive"
                initial={{ scale: 1, opacity: 0.6 }}
                animate={{ scale: 1.5, opacity: 0 }}
                transition={{ duration: 0.4, ease: "easeOut" }}
              />
            )}
          </AnimatePresence>
        </div>
      </div>

      {/* Notes — full width, takes remaining space */}
      <div className="flex-1 flex flex-col min-h-0 relative">
        {isCompact && isRecording && (
          <div className="absolute top-3 right-3 z-10">
            <CompactRecordingIndicator />
          </div>
        )}
        <textarea
          ref={notesRef}
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          placeholder="Take notes during the meeting..."
          className="flex-1 w-full bg-transparent p-6 text-sm leading-relaxed resize-none focus:outline-none placeholder:text-muted-foreground/40"
          autoFocus
        />
      </div>

      <ScratchPad meetingId={currentMeeting?.id ?? null} elapsedMs={elapsed * 1000} />

      {/* Collapsible live transcript */}
      <div className="border-t">
        <button
          onClick={() => setTranscriptOpen((v) => !v)}
          className="flex w-full items-center gap-2 px-6 py-2.5 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
        >
          {transcriptOpen ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
          Live Transcript
          {segments.length > 0 && (
            <Badge variant="secondary" size="sm">
              {segments.length}
            </Badge>
          )}
        </button>
        <Collapsible open={transcriptOpen}>
          {transcriptOpen && (
            <ScrollArea className="h-[200px] border-t">
                <div ref={scrollRef} className="px-6 py-3 space-y-1.5">
                  {segments.length === 0 && transcriptionStatus?.available === false && (
                    <div className="text-xs text-muted-foreground italic">
                      <p>{transcriptionStatus.reason}</p>
                      <p className="mt-1">
                        <Link
                          to="/settings"
                          className="text-primary underline underline-offset-2"
                        >
                          Download models in Settings
                        </Link>{" "}
                        to enable live transcription.
                      </p>
                    </div>
                  )}
                  {segments.length === 0 && transcriptionStatus?.available !== false && (
                    <p className="text-xs text-muted-foreground italic">
                      Ears perked — words will appear as people talk
                    </p>
                  )}
                  {segments.length > 0 &&
                    segments.map((seg) => (
                      <div key={seg.id} className="text-xs">
                        <span className="font-medium text-primary">
                          {seg.speaker_label}:
                        </span>{" "}
                        <span className="text-foreground">{seg.text}</span>
                      </div>
                    ))}
                </div>
              </ScrollArea>
          )}
        </Collapsible>
      </div>
    </div>
  );
}
