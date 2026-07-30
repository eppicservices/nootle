import { useCallback, useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface CopyButtonProps {
  text: string;
  /** `icon` is a bare icon affordance; `button` shows a labelled button. */
  variant?: "icon" | "button";
  /** Label for the `button` variant. */
  label?: string;
  className?: string;
}

/** Copies `text` and confirms with a checkmark. Used for transcripts, summaries, and config snippets. */
export function CopyButton({
  text,
  variant = "icon",
  label = "Copy",
  className,
}: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      return;
    }
    setCopied(true);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setCopied(false), 1500);
  }, [text]);

  if (variant === "button") {
    return (
      <Button
        variant="secondary"
        size="xs"
        onClick={handleCopy}
        className={className}
      >
        {copied ? <Check className="text-success-foreground" /> : <Copy />}
        {copied ? "Copied" : label}
      </Button>
    );
  }

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={copied ? "Copied" : "Copy to clipboard"}
      aria-label={copied ? "Copied" : "Copy to clipboard"}
      className={cn(
        "inline-flex items-center gap-1 rounded text-muted-foreground transition-colors hover:text-foreground",
        className,
      )}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-success-foreground" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
    </button>
  );
}
