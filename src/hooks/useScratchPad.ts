import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ScratchNote } from "@/types";

export function useScratchPad(meetingId: string | null) {
  const [notes, setNotes] = useState<ScratchNote[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!meetingId) {
      setNotes([]);
      setLoading(false);
      return;
    }
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ScratchNote[]>("get_scratch_notes", {
        meetingId,
      });
      setNotes(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [meetingId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const addNote = useCallback(
    async (content: string, timestampMs: number) => {
      if (!meetingId) return;
      await invoke<ScratchNote>("add_scratch_note", {
        meetingId,
        content,
        timestampMs,
      });
      await refresh();
    },
    [meetingId, refresh],
  );

  const deleteNote = useCallback(
    async (id: string) => {
      await invoke("delete_scratch_note", { id });
      await refresh();
    },
    [refresh],
  );

  return { notes, loading, error, addNote, deleteNote, refresh };
}
