import {
  Zap,
  Search,
  Layers,
  Combine,
  Image as ImageIcon,
  Keyboard,
  Lock,
  Sparkles,
  MousePointerClick,
} from "lucide-react";
import type { ReactNode } from "react";

type Feature = {
  icon: ReactNode;
  title: string;
  body: string;
  highlight?: boolean;
};

const FEATURES: Feature[] = [
  {
    icon: <Zap className="h-5 w-5" />,
    title: "Instant palette",
    body: "Hit the hotkey, type a few letters, hit Enter. What you copied is now in front of you — before you even finish typing.",
    highlight: true,
  },
  {
    icon: <Search className="h-5 w-5" />,
    title: "Fuzzy full-text search",
    body: "Find any clip by content, source app, date, or tag. Even partial matches. Even typos.",
  },
  {
    icon: <Layers className="h-5 w-5" />,
    title: "Clipboard Ring",
    body: "Cycle through the last N copies with a second hotkey, like a quick-access stack you rotate through.",
  },
  {
    icon: <Combine className="h-5 w-5" />,
    title: "Merge & paste",
    body: "Select multiple clips and paste them as one. Configurable separator. Great for emails, code snippets, lists.",
  },
  {
    icon: <ImageIcon className="h-5 w-5" />,
    title: "Text, images, files",
    body: "Captures text, images, and file references. Previews generated locally. Nothing leaves your machine.",
  },
  {
    icon: <Keyboard className="h-5 w-5" />,
    title: "Keyboard-first",
    body: "Navigate, select, multi-pick, jump-to-top, jump-to-bottom. Every action has a key — your hands stay on the home row.",
  },
  {
    icon: <MousePointerClick className="h-5 w-5" />,
    title: "Auto-paste",
    body: "Pick a clip and it lands in the previous app automatically. No more tab-switching dance.",
  },
  {
    icon: <Lock className="h-5 w-5" />,
    title: "Local-only by default",
    body: "No accounts, no cloud, no telemetry. Your clipboard history is yours. Period.",
  },
  {
    icon: <Sparkles className="h-5 w-5" />,
    title: "Themes & polish",
    body: "Dark, light, graphite. Sensible defaults. Smooth animations. A native-feeling Windows app, not a webpage wrapper.",
  },
];

export function Features() {
  return (
    <section id="features" className="relative py-20 sm:py-28">
      <div className="container-page">
        <div className="mx-auto max-w-2xl text-center">
          <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-accent">
            Features
          </p>
          <h2 className="text-balance text-3xl font-bold tracking-tight sm:text-5xl">
            Everything you'd want,
            <br className="hidden sm:block" /> nothing you don't.
          </h2>
          <p className="mt-4 text-pretty text-base text-fg-muted sm:text-lg">
            Nine things ClipVault does really well — and a hundred things it intentionally
            doesn't do.
          </p>
        </div>

        <div className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {FEATURES.map((f, i) => (
            <FeatureCard key={f.title} feature={f} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}

function FeatureCard({ feature, index }: { feature: Feature; index: number }) {
  return (
    <div
      className={`card group ${
        feature.highlight
          ? "border-accent/40 bg-gradient-to-br from-accent/10 to-transparent"
          : ""
      }`}
      style={{
        animation: "fadeInUp 0.6s ease-out both",
        animationDelay: `${index * 60}ms`,
      }}
    >
      <div
        className={`mb-4 inline-flex h-10 w-10 items-center justify-center rounded-lg transition-transform duration-300 group-hover:scale-110 ${
          feature.highlight
            ? "bg-accent text-white shadow-lg shadow-accent/30"
            : "bg-bg-overlay text-accent"
        }`}
      >
        {feature.icon}
      </div>
      <h3 className="text-base font-semibold tracking-tight text-fg">{feature.title}</h3>
      <p className="mt-1.5 text-sm leading-relaxed text-fg-muted">{feature.body}</p>
      <div className="absolute inset-x-0 -bottom-px h-px bg-gradient-to-r from-transparent via-accent/40 to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100" />
    </div>
  );
}
