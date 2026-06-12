import { Download, Keyboard, MousePointerClick } from "lucide-react";

const STEPS = [
  {
    icon: <Download className="h-5 w-5" />,
    title: "Install",
    body: "Grab the .exe, double-click, done. No account, no service, no admin required for the user-level install.",
  },
  {
    icon: <Keyboard className="h-5 w-5" />,
    title: "Copy as usual",
    body: "Anything you Ctrl+C in any app shows up in the timeline. ClipVault stays out of your way until you need it.",
  },
  {
    icon: <MousePointerClick className="h-5 w-5" />,
    title: "Paste with one keystroke",
    body: "Hit Ctrl+Shift+V, type a few letters, Enter. Your clip is back in the app you were just in.",
  },
];

export function HowItWorks() {
  return (
    <section className="relative border-y border-border bg-bg-elevated/30 py-20 sm:py-24">
      <div className="container-page">
        <div className="mx-auto max-w-2xl text-center">
          <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-accent">
            How it works
          </p>
          <h2 className="text-balance text-3xl font-bold tracking-tight sm:text-4xl">
            Three steps. Forever.
          </h2>
        </div>
        <ol className="mx-auto mt-12 grid max-w-4xl gap-6 sm:grid-cols-3">
          {STEPS.map((s, i) => (
            <li
              key={s.title}
              className="relative"
              style={{
                animation: "fadeInUp 0.6s ease-out both",
                animationDelay: `${i * 100}ms`,
              }}
            >
              <div className="card h-full">
                <div className="mb-3 flex items-center gap-2">
                  <span className="grid h-6 w-6 place-items-center rounded-full bg-accent text-[11px] font-bold text-white">
                    {i + 1}
                  </span>
                  <span className="text-fg-muted">{s.icon}</span>
                </div>
                <h3 className="text-base font-semibold">{s.title}</h3>
                <p className="mt-1.5 text-sm leading-relaxed text-fg-muted">{s.body}</p>
              </div>
              {i < STEPS.length - 1 && (
                <div className="absolute -right-3 top-1/2 hidden h-px w-6 -translate-y-1/2 bg-gradient-to-r from-border to-transparent sm:block" />
              )}
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
