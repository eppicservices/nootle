import { useEffect, useRef, useState } from "react";
import { NavLink, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import { useTheme } from "@/hooks/useTheme";
import { useAppVersion } from "@/hooks/useAppVersion";
import { useGlobalLLMSelection } from "@/contexts/LLMSelectionContext";
import { useCompactMode } from "@/contexts/CompactModeContext";
import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { MotionButton } from "@/components/MotionButton";
import { NootleLogo } from "@/components/NootleLogo";
import { Mic, Settings, HelpCircle, Circle, Moon, Sun, Lightbulb, MessageSquare, FileText, Bot, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import type { LucideIcon } from "lucide-react";

const navItems: { to: string; label: string; icon: LucideIcon }[] = [
  { to: "/", label: "Meetings", icon: Mic },
  { to: "/insights", label: "Insights", icon: Lightbulb },
  { to: "/chat", label: "Chat", icon: MessageSquare },
  { to: "/templates", label: "Automations", icon: FileText },
  { to: "/settings", label: "Settings", icon: Settings },
  { to: "/help", label: "Help", icon: HelpCircle },
];

export function Sidebar() {
  const navigate = useNavigate();
  const { theme, toggleTheme } = useTheme();
  const version = useAppVersion();
  const { selectedProvider, selectedModel, providers, models, filteredModels, changeProvider, setSelectedModel } = useGlobalLLMSelection();
  const { isCompact, isAutoCompact, toggleCollapsed } = useCompactMode();
  const [wiggleSidebar, setWiggleSidebar] = useState(false);
  const clickCountRef = useRef(0);
  const clickTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );

  useEffect(() => {
    return () => {
      if (clickTimerRef.current) {
        clearTimeout(clickTimerRef.current);
      }
    };
  }, []);

  const handleLogoClick = () => {
    clickCountRef.current += 1;
    if (clickTimerRef.current) {
      clearTimeout(clickTimerRef.current);
    }
    clickTimerRef.current = setTimeout(() => {
      clickCountRef.current = 0;
    }, 1000);

    if (clickCountRef.current >= 5) {
      clickCountRef.current = 0;
      setWiggleSidebar(true);
      setTimeout(() => setWiggleSidebar(false), 500);
    }
  };

  return (
    <motion.aside
      className={cn("group/sidebar flex h-screen flex-col bg-sidebar backdrop-blur-xl backdrop-saturate-[1.8] shadow-[1px_0_0_0_var(--sidebar-border)] transition-[width] duration-200 ease-out", isCompact ? "w-12" : "w-60")}
      animate={
        wiggleSidebar
          ? {
              x: [0, -2, 3, -3, 2, -1, 0],
            }
          : {}
      }
      transition={{ duration: 0.4, ease: "easeInOut" }}
    >
      {/* Logo */}
      <div data-tauri-drag-region className={cn("relative flex items-center px-5 pt-10 pb-4", isCompact ? "justify-center px-2" : "gap-2")}>
        <motion.div
          className="cursor-pointer"
          whileHover={{ rotate: [0, -3, 3, 0] }}
          transition={{ duration: 0.4, ease: "easeInOut" }}
          onClick={handleLogoClick}
        >
          <NootleLogo className="h-8 w-8 text-primary" aria-label="Nootle" />
        </motion.div>
        {!isCompact && <span className="text-lg font-semibold tracking-tight">Nootle</span>}
        {!isAutoCompact && (
          <button
            onClick={toggleCollapsed}
            className={cn(
              "rounded-md p-1 text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors",
              isCompact ? "absolute right-1 top-10 opacity-0 group-hover/sidebar:opacity-100 transition-opacity" : "ml-auto",
            )}
            title={isCompact ? "Expand sidebar" : "Collapse sidebar"}
          >
            {isCompact ? <PanelLeftOpen className="h-4 w-4" /> : <PanelLeftClose className="h-4 w-4" />}
          </button>
        )}
      </div>

      {/* New Recording Button */}
      <div className="px-3 pb-2">
        <MotionButton
          className={isCompact ? "w-full aspect-square justify-center" : "w-full justify-start gap-2"}
          onClick={() => navigate("/recording")}
          title={isCompact ? "Start recording" : undefined}
        >
          <Circle className="h-4 w-4" />
          {!isCompact && "Record Something"}
        </MotionButton>
      </div>

      <Separator />

      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 px-3 pt-3">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === "/"}
            title={isCompact ? item.label : undefined}
            className={({ isActive }) =>
              cn(
                "flex items-center rounded-md transition-colors",
                isCompact ? "justify-center px-2 py-2" : "gap-3 px-3 py-2",
                "text-sm font-medium",
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
              )
            }
          >
            <motion.span
              className="inline-flex"
              whileHover={{ y: -1 }}
              transition={{ type: "spring", stiffness: 300, damping: 10 }}
            >
              <item.icon className="h-4 w-4" />
            </motion.span>
            {!isCompact && item.label}
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      {!isCompact && (
        <div className="space-y-2 px-3 pb-4">
          <Popover>
            <PopoverTrigger asChild>
              <button className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors">
                <Bot className="h-3.5 w-3.5 shrink-0" />
                <span className="truncate">
                  {selectedModel
                    ? models.find((m) => m.id === selectedModel)?.name ?? selectedModel
                    : "No model selected"}
                </span>
              </button>
            </PopoverTrigger>
            <PopoverContent side="top" align="start" className="w-56 p-2">
              <div className="space-y-2">
                <Select
                  size="xs"
                  containerClassName="w-full"
                  value={selectedProvider}
                  onChange={(e) => changeProvider(e.target.value)}
                  aria-label="LLM provider"
                >
                  <option value="">Provider</option>
                  {providers.map((p) => (
                    <option key={p} value={p}>{p}</option>
                  ))}
                </Select>
                <Select
                  size="xs"
                  containerClassName="w-full"
                  value={selectedModel}
                  onChange={(e) => setSelectedModel(e.target.value)}
                  aria-label="LLM model"
                >
                  <option value="">Model</option>
                  {filteredModels.map((m) => (
                    <option key={m.id} value={m.id}>{m.name}</option>
                  ))}
                </Select>
              </div>
            </PopoverContent>
          </Popover>
          <div className="flex items-center justify-between px-2">
            <p className="text-xs text-muted-foreground">v{version}</p>
            <Button
              variant="ghost"
              size="sm"
              onClick={toggleTheme}
              title={theme === "light" ? "Switch to dark mode" : "Switch to light mode"}
              className="h-8 w-8 p-0"
            >
              {theme === "light" ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
            </Button>
          </div>
        </div>
      )}
    </motion.aside>
  );
}
