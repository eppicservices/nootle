import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon: LucideIcon;
  title?: string;
  description: ReactNode;
  /** A primary action button, rendered under the description. */
  action?: ReactNode;
  /** `page` fills the viewport; `panel` suits tabs and side panels. */
  size?: "page" | "panel";
  className?: string;
}

/** The "nothing here yet" block, sized consistently wherever it appears. */
export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  size = "page",
  className,
}: EmptyStateProps) {
  const isPage = size === "page";

  return (
    <div
      className={cn(
        "flex flex-1 flex-col items-center justify-center gap-3 text-center",
        isPage ? "p-12" : "p-8",
        className,
      )}
    >
      <Icon
        className={cn("text-muted-foreground", isPage ? "h-10 w-10" : "h-8 w-8")}
      />
      {title && <h2 className="text-lg font-medium">{title}</h2>}
      <p className="max-w-sm text-sm text-muted-foreground">{description}</p>
      {action}
    </div>
  );
}
