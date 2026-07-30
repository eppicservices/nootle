import {
  AlertTriangle,
  CalendarClock,
  HelpCircle,
  Lightbulb,
  ListChecks,
  MessageSquare,
  Star,
  Target,
  type LucideIcon,
} from "lucide-react";

/**
 * Icons an insight type can use. The keys are the slugs stored in
 * `insight_types.icon`, so this is the single source of truth for both the
 * picker in Settings and everywhere insights are rendered.
 */
export const INSIGHT_ICONS: { value: string; label: string; icon: LucideIcon }[] = [
  { value: "lightbulb", label: "Lightbulb", icon: Lightbulb },
  { value: "list-checks", label: "Checklist", icon: ListChecks },
  { value: "star", label: "Star", icon: Star },
  { value: "alert-triangle", label: "Warning", icon: AlertTriangle },
  { value: "target", label: "Target", icon: Target },
  { value: "calendar-clock", label: "Deadline", icon: CalendarClock },
  { value: "message-square", label: "Discussion", icon: MessageSquare },
  { value: "help-circle", label: "Question", icon: HelpCircle },
];

const BY_SLUG = new Map(INSIGHT_ICONS.map((entry) => [entry.value, entry.icon]));

/** Resolves an insight type's icon slug, falling back to the lightbulb. */
export function insightIcon(slug: string): LucideIcon {
  return BY_SLUG.get(slug) ?? Lightbulb;
}
