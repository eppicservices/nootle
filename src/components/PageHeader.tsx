import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface PageHeaderProps {
  title: string;
  description?: string;
  /** Controls, filters, or actions pinned to the right of the header. */
  actions?: ReactNode;
  className?: string;
}

/** Title block every top-level page starts with, so headers line up across pages. */
export function PageHeader({ title, description, actions, className }: PageHeaderProps) {
  return (
    <div
      className={cn(
        "flex shrink-0 items-start justify-between gap-4 border-b px-6 py-4",
        className,
      )}
    >
      <div className="min-w-0">
        <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
        {description && (
          <p className="mt-1 text-sm text-muted-foreground">{description}</p>
        )}
      </div>
      {actions && (
        <div className="flex shrink-0 items-center gap-3 pt-1">{actions}</div>
      )}
    </div>
  );
}
