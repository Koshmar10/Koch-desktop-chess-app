import { useEffect, useState } from "react";
import { syncChessCom } from "../../api/game";
import { periodStart, type Period } from "../../lib/period";
import GameCardsSection from "./GameCardsSection";
import GamePlaytimeCalendar from "./GamePlaytimeCalendar";
import HistoryToolbar, { type SyncNote } from "./HistoryToolbar";
import ImportPgnPopup from "./ImportPgnPopup";

// Long enough that typing a name searches once, not once per letter.
const SEARCH_DEBOUNCE_MS = 300;

const syncNoteText = (imported: number, failed: number): string => {
  const games = imported === 1 ? "1 new game" : `${imported} new games`;
  const base = imported === 0 ? "Up to date" : games;
  return failed === 0 ? base : `${base}, ${failed} couldn't be read`;
};

const History = () => {
  const [reloadKey, setReloadKey] = useState(0);
  const [search, setSearch] = useState("");
  const [searched, setSearched] = useState("");
  const [period, setPeriod] = useState<Period>("All Time");
  // Here rather than in the toolbar: an empty history offers both too.
  const [importOpen, setImportOpen] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncNote, setSyncNote] = useState<SyncNote | null>(null);

  useEffect(() => {
    const timer = setTimeout(() => setSearched(search), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  const reload = () => setReloadKey((key) => key + 1);

  const handleSync = () => {
    setSyncing(true);
    setSyncNote(null);
    syncChessCom()
      .then(({ imported, failed }) => {
        setSyncNote({ text: syncNoteText(imported, failed), failed: false });
        if (imported > 0) reload();
      })
      .catch((err) => setSyncNote({ text: String(err), failed: true }))
      .finally(() => setSyncing(false));
  };

  const playedFrom = periodStart(period);

  return (
    <div className="w-full h-full flex flex-col">
      <GamePlaytimeCalendar />
      <HistoryToolbar
        search={search}
        onSearchChange={setSearch}
        period={period}
        onPeriodChange={setPeriod}
        syncing={syncing}
        syncNote={syncNote}
        onSync={handleSync}
        onImport={() => setImportOpen(true)}
      />
      {/* Keyed by the filter, so a new search starts again from page 1. */}
      <GameCardsSection
        key={`${searched}|${period}`}
        search={searched}
        playedFrom={playedFrom}
        reloadKey={reloadKey}
        syncing={syncing}
        onSync={handleSync}
        onImport={() => setImportOpen(true)}
      />
      <ImportPgnPopup
        open={importOpen}
        onClose={() => setImportOpen(false)}
        onImported={reload}
      />
    </div>
  );
};

export default History;
