import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChatMessage } from "@/types";

export function useChat(meetingId: string) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const latestMessagesRef = useRef<ChatMessage[]>(messages);
  latestMessagesRef.current = messages;

  // Reset chat when switching to a different meeting so messages from a
  // previous meeting don't leak into the new one.
  useEffect(() => {
    setMessages([]);
    setError(null);
  }, [meetingId]);

  const sendMessage = useCallback(
    async (message: string, provider: string, model: string) => {
      const userMsg: ChatMessage = { role: "user", content: message };
      setMessages((prev) => [...prev, userMsg]);
      setLoading(true);
      setError(null);

      try {
        const response = await invoke<string>("chat_with_meeting", {
          meetingId,
          message,
          history: [...latestMessagesRef.current, userMsg],
          provider,
          model,
        });
        const assistantMsg: ChatMessage = {
          role: "assistant",
          content: response,
        };
        setMessages((prev) => [...prev, assistantMsg]);
        return response;
      } catch (err) {
        setError(String(err));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [meetingId],
  );

  const clearMessages = useCallback(() => {
    setMessages([]);
    setError(null);
  }, []);

  return { messages, loading, error, sendMessage, clearMessages };
}
