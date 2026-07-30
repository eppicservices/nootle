import * as React from "react"
import { ChevronDown } from "lucide-react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const selectVariants = cva(
  "w-full appearance-none rounded-md border border-input bg-transparent text-foreground shadow-xs outline-none transition-[color,box-shadow] dark:bg-input/30 focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 [&>option]:bg-popover [&>option]:text-popover-foreground",
  {
    variants: {
      size: {
        default: "h-9 pl-3 pr-8 text-sm",
        sm: "h-8 pl-2.5 pr-7 text-sm",
        xs: "h-7 pl-2 pr-6 text-xs",
      },
    },
    defaultVariants: {
      size: "default",
    },
  }
)

/**
 * Native `<select>` styled to match `Input`. Width comes from the wrapper, so
 * pass layout classes (`w-full`, `flex-1`, `w-40`) as `containerClassName`.
 */
function Select({
  className,
  containerClassName,
  size = "default",
  children,
  ...props
}: Omit<React.ComponentProps<"select">, "size"> &
  VariantProps<typeof selectVariants> & { containerClassName?: string }) {
  return (
    <div
      className={cn(
        "relative inline-flex shrink-0 items-center",
        containerClassName
      )}
    >
      <select
        data-slot="select"
        className={cn(selectVariants({ size }), className)}
        {...props}
      >
        {children}
      </select>
      <ChevronDown
        aria-hidden
        className={cn(
          "pointer-events-none absolute text-muted-foreground opacity-60",
          size === "xs" ? "right-1.5 size-3" : "right-2.5 size-3.5"
        )}
      />
    </div>
  )
}

export { Select, selectVariants }
