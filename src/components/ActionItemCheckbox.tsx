import { Check } from "lucide-react";

import { cn } from "@/lib/utils";

interface ActionItemCheckboxProps {
  done: boolean;
  onToggle: () => void;
  /** Used for the accessible label, e.g. the action item's text. */
  label: string;
  className?: string;
}

/** The done/open toggle on an action item, shared by the dashboard and meeting detail. */
export function ActionItemCheckbox({
  done,
  onToggle,
  label,
  className,
}: ActionItemCheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={done}
      aria-label={`Mark "${label.slice(0, 60)}" as ${done ? "open" : "done"}`}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      className={cn(
        "mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50",
        done
          ? "border-primary bg-primary text-primary-foreground"
          : "border-muted-foreground hover:border-primary",
        className,
      )}
    >
      {done && <Check className="h-3 w-3" />}
    </button>
  );
}
