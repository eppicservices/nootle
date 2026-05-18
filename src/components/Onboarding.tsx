import { useState, useMemo, useEffect, useCallback, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { MotionButton } from "@/components/MotionButton";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  useModelDownload,
  MODELS_REQUIRING_AUTH,
  type ModelDefinition,
} from "@/hooks/useModelDownload";
import { Mic, Monitor, Calendar } from "lucide-react";
import { formatBytes } from "@/lib/utils";

const STEPS = ["Welcome", "Permissions", "Models", "API Keys", "Done"] as const;
type Step = (typeof STEPS)[number];

const PROVIDERS = [
  { id: "openai", name: "OpenAI", placeholder: "sk-..." },
  { id: "anthropic", name: "Anthropic", placeholder: "sk-ant-..." },
  { id: "google", name: "Google Gemini", placeholder: "AIza..." },
  { id: "groq", name: "Groq", placeholder: "gsk_..." },
  { id: "openrouter", name: "OpenRouter", placeholder: "sk-or-..." },
] as const;

function SparkleShower() {
  const particles = Array.from({ length: 12 }, (_, i) => ({
    id: i,
    left: Math.random() * 100,
    delay: Math.random() * 2,
    duration: 2 + Math.random() * 2,
    size: 2 + Math.random() * 3,
  }));

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {particles.map((p) => (
        <motion.div
          key={p.id}
          className="absolute rounded-full bg-primary/30"
          style={{
            left: `${p.left}%`,
            width: p.size,
            height: p.size,
          }}
          initial={{ y: -10, opacity: 0 }}
          animate={{ y: "100%", opacity: [0, 0.8, 0] }}
          transition={{
            duration: p.duration,
            delay: p.delay,
            repeat: Infinity,
            ease: "linear",
          }}
        />
      ))}
    </div>
  );
}

export function Onboarding({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState<Step>("Welcome");
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  const stepIndex = STEPS.indexOf(step);

  const next = () => {
    const nextIndex = stepIndex + 1;
    if (nextIndex < STEPS.length) {
      setStep(STEPS[nextIndex]);
    }
  };

  const finish = async () => {
    setSaving(true);
    try {
      for (const [provider, key] of Object.entries(apiKeys)) {
        if (key.trim()) {
          await invoke("store_api_key", { provider, key: key.trim() });
        }
      }
      setApiKeys({});
      await invoke("seed_default_prompts");
      localStorage.setItem("onboarding_complete", "true");
      onComplete();
    } catch (e) {
      console.error("Onboarding error:", e);
      setApiKeys({});
      // Still complete even if seeding fails
      localStorage.setItem("onboarding_complete", "true");
      onComplete();
    } finally {
      setSaving(false);
    }
  };

  return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-background">
        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{ duration: 0.3 }}
            className="mx-auto w-full max-w-lg overflow-y-auto max-h-[90vh] rounded-2xl border border-border bg-card p-8 shadow-2xl"
          >
            {/* Progress dots */}
            <div className="mb-8 flex justify-center gap-2">
              {STEPS.map((s, i) => (
                <div
                  key={s}
                  className={`h-2 w-2 rounded-full transition-colors ${
                    i <= stepIndex ? "bg-primary" : "bg-muted"
                  }`}
                />
              ))}
            </div>

            {step === "Welcome" && (
              <div className="text-center">
                <h1 className="mb-2 text-3xl font-bold text-foreground">
                  Welcome to Nootle
                </h1>
                <p className="mb-8 text-muted-foreground">
                  Nootle captures your meetings, transcribes them live, and
                  cooks up smart summaries — all on your Mac.
                </p>
                <MotionButton size="lg" onClick={next}>
                  Get Started
                </MotionButton>
              </div>
            )}

            {step === "Permissions" && (
              <PermissionsStep onNext={next} />
            )}

            {step === "Models" && <ModelsStep onNext={next} />}

            {step === "API Keys" && (
              <div>
                <h2 className="mb-2 text-2xl font-bold text-foreground">
                  AI Providers
                </h2>
                <p className="mb-6 text-sm text-muted-foreground">
                  Add API keys for AI-powered summaries and chat. You can skip
                  this and add them later in Settings. Using Ollama? No key
                  needed — Nootle auto-detects it.
                </p>
                <div className="space-y-4">
                  {PROVIDERS.map((p) => (
                    <div key={p.id}>
                      <label className="mb-1 block text-sm font-medium text-foreground">
                        {p.name}
                      </label>
                      <Input
                        type="password"
                        placeholder={p.placeholder}
                        value={apiKeys[p.id] || ""}
                        onChange={(e) =>
                          setApiKeys((prev) => ({
                            ...prev,
                            [p.id]: e.target.value,
                          }))
                        }
                      />
                    </div>
                  ))}
                </div>
                <div className="mt-8 flex justify-between">
                  <Button variant="ghost" onClick={next}>
                    Skip
                  </Button>
                  <MotionButton onClick={next}>Continue</MotionButton>
                </div>
              </div>
            )}

            {step === "Done" && (
              <div className="relative text-center">
                <SparkleShower />
                <h2 className="relative mb-2 text-3xl font-bold text-foreground">
                  You're ready to nootle!
                </h2>
                <p className="relative mb-8 text-muted-foreground">
                  Nootle will automatically detect meetings in Zoom, Teams, and
                  Google Meet. You can also start recording manually anytime.
                </p>
                <MotionButton size="lg" onClick={finish} disabled={saving} className="relative">
                  {saving ? "Setting up..." : "Start Using Nootle"}
                </MotionButton>
              </div>
            )}
          </motion.div>
        </AnimatePresence>
      </div>
  );
}

interface PermissionStatusData {
  microphone: string;
  screen_recording: boolean;
  calendar: string;
}

function PermissionsStep({ onNext }: { onNext: () => void }) {
  const [status, setStatus] = useState<PermissionStatusData | null>(null);
  const [requesting, setRequesting] = useState<string | null>(null);

  const checkStatus = useCallback(async () => {
    try {
      const s = await invoke<PermissionStatusData>("check_permissions");
      setStatus(s);
    } catch (e) {
      console.error("Failed to check permissions:", e);
    }
  }, []);

  useEffect(() => {
    checkStatus();
    const interval = setInterval(checkStatus, 2000);
    return () => clearInterval(interval);
  }, [checkStatus]);

  const allGranted =
    status?.microphone === "granted" &&
    status?.screen_recording === true &&
    status?.calendar === "granted";

  const requestPermission = async (type: "microphone" | "screen_recording" | "calendar") => {
    setRequesting(type);
    try {
      if (type === "microphone") {
        await invoke("request_microphone_permission");
      } else if (type === "screen_recording") {
        await invoke("request_screen_recording_permission");
      } else {
        await invoke("request_calendar_permission");
      }
      await checkStatus();
    } catch (e) {
      console.error(`Failed to request ${type} permission:`, e);
    } finally {
      setRequesting(null);
    }
  };

  const micGranted = status?.microphone === "granted";
  const screenGranted = status?.screen_recording === true;
  const calGranted = status?.calendar === "granted";

  return (
    <div>
      <h2 className="mb-2 text-2xl font-bold text-foreground">Permissions</h2>
      <p className="mb-6 text-sm text-muted-foreground">
        Nootle needs these permissions to record and transcribe your meetings.
      </p>
      <div className="space-y-3">
        <PermissionRow
          icon={<Mic className="h-5 w-5" />}
          title="Microphone"
          desc="Record your voice during meetings"
          granted={micGranted}
          onRequest={() => requestPermission("microphone")}
          requesting={requesting === "microphone"}
        />
        <PermissionRow
          icon={<Monitor className="h-5 w-5" />}
          title="Screen Recording"
          desc="Capture system audio from meeting apps"
          granted={screenGranted}
          onRequest={() => requestPermission("screen_recording")}
          requesting={requesting === "screen_recording"}
          buttonLabel={screenGranted ? undefined : "Open System Settings"}
        />
        <PermissionRow
          icon={<Calendar className="h-5 w-5" />}
          title="Calendar"
          desc="Auto-detect meetings from your calendar"
          granted={calGranted}
          onRequest={() => requestPermission("calendar")}
          requesting={requesting === "calendar"}
        />
      </div>
      {!allGranted && (
        <p className="mt-4 text-xs text-muted-foreground">
          Screen Recording requires toggling in System Settings. It will be detected automatically.
        </p>
      )}
      <div className="mt-8 flex justify-end">
        <MotionButton onClick={onNext} disabled={!allGranted}>
          Continue
        </MotionButton>
      </div>
    </div>
  );
}

function ModelsStep({ onNext }: { onNext: () => void }) {
  const { registry: fullRegistry, diskStatus, progress, downloadModel, cancelDownload } =
    useModelDownload();
  const registry = useMemo(
    () => fullRegistry.filter((m) => !MODELS_REQUIRING_AUTH.has(m.id)),
    [fullRegistry],
  );

  // Track selected variant per model (default to "int8" for models that have it)
  const [selectedVariants, setSelectedVariants] = useState<
    Record<string, string>
  >({});
  const [downloadingAll, setDownloadingAll] = useState(false);
  const [modelErrors, setModelErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (
      progress &&
      typeof progress.state === "object" &&
      "error" in progress.state
    ) {
      const message = progress.state.error.message;
      const modelId = progress.model_id;
      setModelErrors((prev) =>
        prev[modelId] === message ? prev : { ...prev, [modelId]: message },
      );
    }
  }, [progress]);

  const variantSelections = useMemo(() => {
    const selections: Record<string, string> = {};
    for (const model of registry) {
      if (selectedVariants[model.id]) {
        selections[model.id] = selectedVariants[model.id];
      } else if (model.variants.length === 1) {
        selections[model.id] = model.variants[0].id;
      } else {
        // Default to "int8" if available, otherwise first variant
        const int8 = model.variants.find((v) => v.id === "int8");
        selections[model.id] = int8 ? "int8" : model.variants[0].id;
      }
    }
    return selections;
  }, [registry, selectedVariants]);

  const allDownloaded = registry.length > 0 && registry.every((model) => {
    const status = diskStatus.find((d) => d.model_id === model.id);
    return status?.downloaded;
  });

  const isDownloading = progress !== null && typeof progress.state === "string" && (progress.state === "downloading" || progress.state === "verifying");

  const handleDownloadAll = async () => {
    setDownloadingAll(true);
    setModelErrors({});
    for (const model of registry) {
      const status = diskStatus.find((d) => d.model_id === model.id);
      if (status?.downloaded) continue;
      const variantId = variantSelections[model.id];
      if (!variantId) continue;
      try {
        await downloadModel(model.id, variantId);
      } catch (e) {
        console.error(`Failed to download ${model.id}:`, e);
        setModelErrors((prev) => ({ ...prev, [model.id]: String(e) }));
      }
    }
    setDownloadingAll(false);
  };

  const getModelStatus = (model: ModelDefinition) => {
    return diskStatus.find((d) => d.model_id === model.id);
  };

  const getProgressState = (): string => {
    if (!progress) return "";
    if (typeof progress.state === "string") return progress.state;
    if (typeof progress.state === "object" && "error" in progress.state)
      return `Error: ${progress.state.error.message}`;
    return "";
  };

  return (
    <div>
      <h2 className="mb-2 text-2xl font-bold text-foreground">AI Models</h2>
      <p className="mb-6 text-sm text-muted-foreground">
        Download local AI models for transcription and speaker identification.
        These run entirely on your Mac for privacy and speed.
      </p>

      <div className="space-y-4">
        {registry.map((model) => {
          const status = getModelStatus(model);
          const isThisModelDownloading =
            isDownloading && progress?.model_id === model.id;

          return (
            <div
              key={model.id}
              className="rounded-lg border border-border p-4"
            >
              <div className="flex items-center justify-between mb-1">
                <span className="font-medium text-foreground">
                  {model.name}
                </span>
                {status?.downloaded && (
                  <Badge
                    variant="secondary"
                    className="bg-green-500/15 text-green-500 text-[10px]"
                  >
                    Downloaded
                  </Badge>
                )}
              </div>
              <p className="text-xs text-muted-foreground mb-3">
                {model.description}
              </p>

              {/* Variant picker for models with multiple variants */}
              {!status?.downloaded && model.variants.length > 1 && (
                <div className="flex gap-3 mb-3">
                  {model.variants.map((variant) => (
                    <label
                      key={variant.id}
                      className="flex items-center gap-2 cursor-pointer"
                    >
                      <input
                        type="radio"
                        name={`variant-${model.id}`}
                        checked={variantSelections[model.id] === variant.id}
                        onChange={() =>
                          setSelectedVariants((prev) => ({
                            ...prev,
                            [model.id]: variant.id,
                          }))
                        }
                        className="accent-primary"
                      />
                      <span className="text-sm text-foreground">
                        {variant.label}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        ({formatBytes(variant.total_size_bytes)})
                      </span>
                    </label>
                  ))}
                </div>
              )}

              {/* Single variant - show size */}
              {!status?.downloaded && model.variants.length === 1 && (
                <p className="text-xs text-muted-foreground mb-3">
                  Size: {formatBytes(model.variants[0].total_size_bytes)}
                </p>
              )}

              {/* Per-model error from a previous Download All attempt */}
              {!status?.downloaded && !isThisModelDownloading && modelErrors[model.id] && (
                <p className="text-xs text-destructive mb-2">
                  Failed: {modelErrors[model.id]}
                </p>
              )}

              {/* Progress bar for this model */}
              {isThisModelDownloading && progress && (
                <div className="mt-2">
                  <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
                    <span>
                      {getProgressState() === "verifying"
                        ? "Verifying..."
                        : `Downloading ${progress.current_file}`}
                    </span>
                    <span>
                      {Math.round(progress.overall_percent * 100)}%
                    </span>
                  </div>
                  <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                    <div
                      className="h-full rounded-full bg-primary transition-[width] duration-300"
                      style={{
                        width: `${Math.round(progress.overall_percent * 100)}%`,
                      }}
                    />
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Global progress and actions */}
      <div className="mt-8 flex justify-between items-center">
        <Button variant="ghost" onClick={onNext}>
          Skip
        </Button>

        <div className="flex gap-2">
          {isDownloading && (
            <Button
              variant="outline"
              onClick={cancelDownload}
            >
              Cancel
            </Button>
          )}

          {allDownloaded ? (
            <Button onClick={onNext}>Continue</Button>
          ) : (
            <Button
              onClick={handleDownloadAll}
              disabled={isDownloading || downloadingAll || registry.length === 0}
            >
              {isDownloading
                ? "Downloading..."
                : "Download All"}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

function PermissionRow({
  icon,
  title,
  desc,
  granted,
  onRequest,
  requesting,
  buttonLabel,
}: {
  icon: ReactNode;
  title: string;
  desc: string;
  granted: boolean;
  onRequest: () => void;
  requesting: boolean;
  buttonLabel?: string;
}) {
  return (
    <div className="flex items-center gap-3 rounded-lg border border-border p-3">
      <span className="text-muted-foreground">{icon}</span>
      <div className="flex-1">
        <p className="font-medium text-foreground">{title}</p>
        <p className="text-sm text-muted-foreground">{desc}</p>
      </div>
      {granted ? (
        <motion.div
          initial={{ scale: 1 }}
          animate={{ scale: [1, 1.15, 1] }}
          transition={{ duration: 0.3 }}
        >
          <Badge variant="secondary" className="bg-green-500/15 text-green-600 dark:text-green-400">
            Granted
          </Badge>
        </motion.div>
      ) : (
        <Button
          variant="outline"
          size="sm"
          onClick={onRequest}
          disabled={requesting}
        >
          {requesting ? "..." : buttonLabel || "Grant"}
        </Button>
      )}
    </div>
  );
}
