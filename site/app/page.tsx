import { Navbar } from "@/components/Navbar";
import { Hero } from "@/components/Hero";
import { Introduction } from "@/components/Introduction";
import { Screenshots } from "@/components/Screenshots";
import { Features } from "@/components/Features";
import { Comparison } from "@/components/Comparison";
import { Integrations } from "@/components/Integrations";
import { HowItWorks } from "@/components/HowItWorks";
import { SystemRequirements } from "@/components/SystemRequirements";
import { FAQ } from "@/components/FAQ";
import { SponsorCTA } from "@/components/SponsorCTA";

export default function Home() {
  return (
    <main id="main-content" className="relative overflow-hidden">

      <Navbar />
      <Hero />
      <Introduction />
      <Screenshots />
      <Features />
      <Comparison />
      <Integrations />
      <HowItWorks />
      <SystemRequirements />
      <FAQ />
      <SponsorCTA />
      <footer className="py-8 text-center text-[var(--color-text-secondary)] text-sm">
        <p>
          Made by{" "}
          <a
            href="https://michellemayes.me"
            className="underline hover:text-[var(--color-text)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] focus-visible:rounded-sm"
          >
            Michelle Mayes
          </a>
          {" "}&middot;{" "}
          <a
            href="https://github.com/michellemayes/nootle"
            className="underline hover:text-[var(--color-text)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] focus-visible:rounded-sm"
          >
            GitHub
          </a>
          {" "}&middot;{" "}
          <a
            href="/privacy"
            className="underline hover:text-[var(--color-text)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] focus-visible:rounded-sm"
          >
            Privacy
          </a>
        </p>
      </footer>
    </main>
  );
}
