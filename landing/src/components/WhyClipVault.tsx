import { Lock, Zap, Eye } from "lucide-react";

const POINTS = [
  {
    icon: <Lock className="h-5 w-5" />,
    title: "Local-only by default",
    body: "Your clipboard history lives in a SQLite file on your disk. No remote calls. No analytics. No 'anonymous' identifiers. Verify it yourself — the data dir is right there.",
  },
  {
    icon: <Zap className="h-5 w-5" />,
    title: "Native, not a webpage",
    body: "Built with Tauri and Rust. Single ~5 MB installer. Starts in under 200 ms. Uses the same APIs Windows itself uses to watch the clipboard.",
  },
  {
    icon: <Eye className="h-5 w-5" />,
    title: "Open source, MIT",
    body: "Every line of ClipVault is on GitHub. Audit it, fork it, self-host a build. No proprietary bits. No telemetry SDKs to disable.",
  },
];

export function WhyClipVault() {
  return (
    <section className="relative py-20 sm:py-28">
      <div className="container-page">
        <div className="grid items-start gap-12 lg:grid-cols-2">
          <div>
            <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-accent">
              Why ClipVault
            </p>
            <h2 className="text-balance text-3xl font-bold tracking-tight sm:text-5xl">
              A clipboard manager
              <br />
              that <span className="gradient-text">respects you</span>.
            </h2>
            <p className="mt-4 max-w-md text-pretty text-base text-fg-muted sm:text-lg">
              Most clipboard apps sync to the cloud, ask for an account, or shove ads
              in your face. ClipVault doesn't, because it doesn't have to — and
              because you shouldn't have to put up with that.
            </p>
            <div className="mt-6 inline-flex items-center gap-2 rounded-full border border-accent/30 bg-accent/10 px-3 py-1.5 text-xs font-medium text-accent">
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-accent opacity-75" />
                <span className="relative inline-flex h-2 w-2 rounded-full bg-accent" />
              </span>
              0 network calls on the happy path
            </div>
          </div>

          <ul className="space-y-4">
            {POINTS.map((p, i) => (
              <li
                key={p.title}
                className="card flex gap-4"
                style={{
                  animation: "fadeInUp 0.6s ease-out both",
                  animationDelay: `${i * 100}ms`,
                }}
              >
                <span className="grid h-10 w-10 shrink-0 place-items-center rounded-lg bg-accent/15 text-accent">
                  {p.icon}
                </span>
                <div>
                  <h3 className="text-base font-semibold">{p.title}</h3>
                  <p className="mt-1 text-sm leading-relaxed text-fg-muted">{p.body}</p>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}
