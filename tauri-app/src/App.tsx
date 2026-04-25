import { useEffect } from "react";
import Sidebar from "./components/Sidebar";
import SearchBar from "./components/SearchBar";
import Timeline from "./components/Timeline";
import MediaViewer from "./components/MediaViewer";
import { useAppStore } from "./stores/appStore";
import { getStats } from "./hooks/useTauri";

function App() {
  const { viewingMedia, setStats } = useAppStore();

  useEffect(() => {
    // Load stats on mount
    getStats()
      .then((stats) => setStats(stats))
      .catch(console.error);
  }, [setStats]);

  return (
    <div className="flex h-screen bg-neutral-900 text-white">
      <Sidebar />
      <div className="flex-1 flex flex-col">
        <SearchBar />
        <Timeline />
      </div>
      {viewingMedia !== null && <MediaViewer />}
    </div>
  );
}

export default App;
