"use client";

import { useState } from "react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";
import { AnimateIn } from "@/components/AnimateIn";

const SHOTS = [
  {
    id: "library",
    label: "Meetings",
    caption: "Every meeting in one library — searchable, labelled, and summarized.",
    alt: "Nootle meeting library showing recorded meetings with labels, dates, and durations",
  },
  {
    id: "recording",
    label: "Recording",
    caption: "Record mic and system audio while the transcript builds in real time.",
    alt: "Nootle recording a meeting with live transcript and notes",
  },
  {
    id: "meeting",
    label: "Transcript & summary",
    caption: "Speaker-labelled transcript beside an AI summary you can jump around in.",
    alt: "Nootle meeting detail with the transcript on the left and an AI summary on the right",
  },
  {
    id: "insights",
    label: "Insights",
    caption: "Decisions, action items, and key moments pulled out of every meeting.",
    alt: "Nootle insights dashboard listing decisions and action items across meetings",
  },
  {
    id: "chat",
    label: "Chat",
    caption: "Ask questions across your whole meeting history, with citations.",
    alt: "Nootle chat answering a question about onboarding drop-off with sources cited",
  },
  {
    id: "automations",
    label: "Automations",
    caption: "Templates and workflows so the right summary runs on its own.",
    alt: "Nootle automations page showing summary templates",
  },
] as const;

export function Screenshots() {
  const [active, setActive] = useState<(typeof SHOTS)[number]["id"]>("library");
  const shot = SHOTS.find((s) => s.id === active) ?? SHOTS[0];

  return (
    <section aria-label="Screenshots" className="py-24 px-6">
      <div className="max-w-6xl mx-auto">
        <AnimateIn>
          <h2 className="font-[family-name:var(--font-outfit)] text-4xl md:text-5xl font-bold text-center mb-4 text-[var(--color-text)]">
            See it in action
          </h2>
          <p className="text-xl text-[var(--color-text-secondary)] text-center mb-10 max-w-2xl mx-auto">
            Record meetings, get transcripts with speaker labels, then let AI surface what matters.
          </p>
        </AnimateIn>

        <div
          role="tablist"
          aria-label="App screenshots"
          className="flex flex-wrap justify-center gap-2 mb-8"
        >
          {SHOTS.map((s) => (
            <button
              key={s.id}
              role="tab"
              aria-selected={s.id === active}
              aria-controls={`screenshot-${s.id}`}
              onClick={() => setActive(s.id)}
              className={`rounded-full px-4 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] ${
                s.id === active
                  ? "bg-[var(--color-accent)] text-white"
                  : "bg-[var(--color-surface)] text-[var(--color-text-secondary)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-elevated)]"
              }`}
            >
              {s.label}
            </button>
          ))}
        </div>

        <AnimateIn delay={0.1} className="relative mx-auto max-w-5xl">
          <AnimatePresence mode="wait">
            <motion.div
              key={shot.id}
              id={`screenshot-${shot.id}`}
              role="tabpanel"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
            >
              <Image
                src={`/screenshots/${shot.id}-dark.png`}
                alt={shot.alt}
                width={1600}
                height={1060}
                priority={shot.id === "library"}
                className="w-full h-auto"
              />
              <p className="mt-4 text-center text-[var(--color-text-secondary)]">
                {shot.caption}
              </p>
            </motion.div>
          </AnimatePresence>

          {/* Layered glow behind the screenshot */}
          <div
            aria-hidden="true"
            className="absolute -inset-8 -z-10 rounded-3xl opacity-20 blur-[60px]"
            style={{ background: "linear-gradient(135deg, var(--color-accent), var(--color-cyan))" }}
          />
        </AnimateIn>
      </div>
    </section>
  );
}
