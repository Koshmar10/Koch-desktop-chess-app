import { useState } from "react";

import { RefreshCcw, Upload } from "lucide-react";
import { syncChessCom } from "../../api/game";
import SearchBox from "../../components/SearchBox";
import SelectMenu from "../../components/SelectMenu";
import TooltipButton from "../../components/TooltipButton";
import { PERIODS, type Period } from "../../lib/period";
import ImportPgnPopup from "./ImportPgnPopup";

// What the last sync did, in a few words beside its button.
interface SyncNote {
  text: string;
  failed: boolean;
}

const syncNoteText = (imported: number, failed: number): string => {
  const games = imported === 1 ? "1 new game" : `${imported} new games`;
  const base = imported === 0 ? "Up to date" : games;
  return failed === 0 ? base : `${base}, ${failed} couldn't be read`;
};

interface HistoryToolbarProps {
  search: string;
  onSearchChange: (search: string) => void;
  period: Period;
  onPeriodChange: (period: Period) => void;
  // After an import or a sync that saved games, so the list fetches again.
  onImported?: () => void;
}

const HistoryToolbar = ({
  search,
  onSearchChange,
  period,
  onPeriodChange,
  onImported,
}: HistoryToolbarProps) => {
  const [importOpen, setImportOpen] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncNote, setSyncNote] = useState<SyncNote | null>(null);

  const handleSync = () => {
    setSyncing(true);
    setSyncNote(null);
    syncChessCom()
      .then(({ imported, failed }) => {
        setSyncNote({ text: syncNoteText(imported, failed), failed: false });
        if (imported > 0) onImported?.();
      })
      .catch((err) => setSyncNote({ text: String(err), failed: true }))
      .finally(() => setSyncing(false));
  };

  return (
    <div className="gap-4 text-foreground border-b-2 border-border flex flex-row items-center p-3">
      <SearchBox
        value={search}
        onChange={onSearchChange}
        placeholder="Search players or openings..."
      />
      <SelectMenu
        options={PERIODS.map((option) => ({ value: option, label: option }))}
        value={period}
        onChange={onPeriodChange}
      />
      <TooltipButton
        icon={RefreshCcw}
        iconClassName={syncing ? "animate-spin" : undefined}
        tooltip={syncing ? "Syncing with chess.com…" : "Sync with chess.com"}
        onClick={handleSync}
        disabled={syncing}
      />
      <TooltipButton
        icon={Upload}
        tooltip="Import PGN"
        onClick={() => setImportOpen(true)}
      />
      {syncNote && (
        <span
          className={`text-sm ${syncNote.failed ? "text-red-400" : "text-foreground/60"}`}
        >
          {syncNote.text}
        </span>
      )}
      <ImportPgnPopup
        open={importOpen}
        onClose={() => setImportOpen(false)}
        onImported={onImported}
      />
    </div>
  );
};

export default HistoryToolbar;
