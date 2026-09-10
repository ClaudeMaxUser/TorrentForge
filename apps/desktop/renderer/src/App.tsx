import React from "react";
import "./App.css";

interface Window {
  electronAPI?: any;
}

const App: React.FC = () => {
  const [torrents, setTorrents] = React.useState<any[]>([]);

  React.useEffect(() => {
    // Fetch torrents on mount
    if ((window as any).electronAPI) {
      (window as any).electronAPI.getTorrents().then(setTorrents);
    }
  }, []);

  return (
    <div className="App">
      <header className="App-header">
        <h1>Torrent Forge</h1>
        <p>Production-grade BitTorrent Client</p>
      </header>

      <main>
        <section className="torrents">
          <h2>Torrents</h2>
          {torrents.length === 0 ? (
            <p>No torrents yet. Add one to get started.</p>
          ) : (
            <ul>
              {torrents.map((t) => (
                <li key={t.id}>{t.name}</li>
              ))}
            </ul>
          )}
        </section>
      </main>
    </div>
  );
};

export default App;
