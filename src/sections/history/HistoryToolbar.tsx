import { RefreshCcw, Upload } from "lucide-react";
import SearchBox from "../../components/SearchBox";
import SelectMenu from "../../components/SelectMenu";
import TooltipButton from "../../components/TooltipButton";
import { PERIODS, type Period } from "../../lib/period";

// What the last sync did, in a few words beside its button.
export interface SyncNote {
  text: string;
  failed: boolean;
}

interface HistoryToolbarProps {
  search: string;
  onSearchChange: (search: string) => void;
  period: Period;
  onPeriodChange: (period: Period) => void;
  syncing: boolean;
  syncNote: SyncNote | null;
  onSync: () => void;
  onImport: () => void;
}

const HistoryToolbar = ({
  search,
  onSearchChange,
  period,
  onPeriodChange,
  syncing,
  syncNote,
  onSync,
  onImport,
}: HistoryToolbarProps) => (
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
      onClick={onSync}
      disabled={syncing}
    />
    <TooltipButton icon={Upload} tooltip="Import PGN" onClick={onImport} />
    {syncNote && (
      <span
        className={`text-sm ${syncNote.failed ? "text-red-400" : "text-foreground/60"}`}
      >
        {syncNote.text}
      </span>
    )}
  </div>
);

export default HistoryToolbar;
