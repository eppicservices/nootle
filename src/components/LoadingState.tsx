import { cn } from "@/lib/utils";

/**
 * Loading copy, kept in one place so the app's voice stays consistent instead
 * of drifting between playful lines and bare "Loading...".
 */
export const LOADING_COPY = {
  meetings: [
    "Warming up the noodles...",
    "Untangling the transcript...",
    "Slurping through the data...",
    "Almost there, just al dente...",
    "Stirring the meeting pot...",
    "Draining the audio linguine...",
  ],
  meeting: "Setting the table...",
  insights: "Fishing out the good bits...",
  transcript: "Unspooling the transcript...",
  analytics: "Tallying up the numbers...",
  templates: "Simmering your templates...",
  slashCommands: "Sharpening the knives...",
  workflows: "Prepping the workflows...",
  integrations: "Checking the pantry...",
  permissions: "Checking permissions...",
} as const;

/** Picks a random line from `LOADING_COPY.meetings`. */
export function randomMeetingsLoadingMessage(): string {
  const messages = LOADING_COPY.meetings;
  return messages[Math.floor(Math.random() * messages.length)];
}

interface LoadingStateProps {
  message: string;
  /** `fill` centers in the remaining space; `inline` sits in normal flow. */
  layout?: "fill" | "inline";
  className?: string;
}

export function LoadingState({
  message,
  layout = "fill",
  className,
}: LoadingStateProps) {
  if (layout === "inline") {
    return (
      <p className={cn("text-sm text-muted-foreground", className)}>{message}</p>
    );
  }

  return (
    <div
      className={cn("flex flex-1 items-center justify-center p-8", className)}
    >
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );
}
