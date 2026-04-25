import { useEffect, useState } from "react";
import Sidebar from "./components/Sidebar";
import SearchBar from "./components/SearchBar";
import Timeline from "./components/Timeline";
import MediaViewer from "./components/MediaViewer";
import StatusBar from "./components/StatusBar";
import PrefsSheet from "./components/PrefsSheet";
import { useAppStore } from "./stores/appStore";
import { usePrefs } from "./stores/prefs";
import { getStats } from "./hooks/useTauri";

function App() {
  const { viewingMedia, setStats } = useAppStore();
  const loadPrefs = usePrefs((s) => s.load);
  const [prefsOpen, setPrefsOpen] = useState(false);

  useEffect(() => {
    loadPrefs();
    getStats()
      .then((stats) => setStats(stats))
      .catch(console.error);
  }, [loadPrefs, setStats]);

  // Global hotkeys: ⌘, opens preferences. Other shortcuts live with
  // the components that own them (viewer, search, etc.).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        setPrefsOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="flex h-screen surface-1 fg-1">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <SearchBar />
        <Timeline />
        <StatusBar />
      </div>
      {viewingMedia !== null && <MediaViewer />}
      <PrefsSheet open={prefsOpen} onClose={() => setPrefsOpen(false)} />
    </div>
  );
}

export default App;
