import { useEffect, useState } from "react";
import * as api from "./api/client";
import { AppProvider } from "./context";
import DesignPage from "./pages/DesignPage";
import SimulatePage from "./pages/SimulatePage";
import RunPage from "./pages/RunPage";

type Tab = "design" | "simulate" | "run";

function Shell() {
  const [tab, setTab] = useState<Tab>("design");
  const [version, setVersion] = useState("…");
  const [estop, setEstop] = useState(false);

  useEffect(() => {
    api.health().then((h) => setVersion(h.version)).catch(() => setVersion("offline"));
    const poll = setInterval(() => {
      api.runStatus().then((s) => setEstop(s.estopped)).catch(() => {});
    }, 500);
    return () => clearInterval(poll);
  }, []);

  return (
    <div className="app">
      <header className="chrome">
        <div className="brand">Spyder</div>
        <nav className="tabs">
          {(["design", "simulate", "run"] as Tab[]).map((t) => (
            <button
              key={t}
              type="button"
              className={`tab ${tab === t ? "active" : ""}`}
              onClick={() => setTab(t)}
            >
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </nav>
      </header>
      <main className="main">
        {tab === "design" && <DesignPage />}
        {tab === "simulate" && <SimulatePage />}
        {tab === "run" && <RunPage />}
      </main>
      <footer className="status-bar">
        <span className="ok">API v{version}</span>
        <span>127.0.0.1:7700</span>
        {estop && <span className="danger">E-STOP ACTIVE</span>}
      </footer>
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <Shell />
    </AppProvider>
  );
}
