import { Nav } from "./components/Nav";
import { Hero } from "./components/Hero";
import { Features } from "./components/Features";
import { HowItWorks } from "./components/HowItWorks";
import { LiveDemo } from "./components/LiveDemo";
import { WhyClipVault } from "./components/WhyClipVault";
import { Install } from "./components/Install";
import { Footer } from "./components/Footer";

export default function App() {
  return (
    <div className="relative min-h-screen bg-bg">
      <Nav />
      <main>
        <Hero />
        <Features />
        <HowItWorks />
        <LiveDemo />
        <WhyClipVault />
        <Install />
      </main>
      <Footer />
    </div>
  );
}
