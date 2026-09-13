import { useState } from "react"
import { AlertTriangle, CheckCircle2, Info } from "lucide-react"
import Popup from "../../components/popup/Popup"
import { importPgn } from "../../api/game"
import { ImportPgnResult } from "../../api/bindings/ImportPgnResult"

interface ImportPgnPopupProps {
  open: boolean
  onClose: () => void
  onImported?: () => void
}

type Outcome =
  | { kind: "error"; text: string }
  | { kind: "duplicate" }
  | { kind: "partial"; result: ImportPgnResult }
  | { kind: "ok"; result: ImportPgnResult }

const outcomeFor = (result: ImportPgnResult): Outcome => {
  if (result.game_id === null) return { kind: "duplicate" }
  if (result.truncated_at_ply !== null) return { kind: "partial", result }
  return { kind: "ok", result }
}

const OutcomeNote = ({ outcome }: { outcome: Outcome }) => {
  switch (outcome.kind) {
    case "error":
      return (
        <p className="flex items-center gap-2 text-sm text-red-500 dark:text-red-400">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          {outcome.text}
        </p>
      )
    case "duplicate":
      return (
        <p className="flex items-center gap-2 text-sm text-foreground/60">
          <Info className="h-4 w-4 shrink-0" />
          This game is already in your library.
        </p>
      )
    case "partial":
      return (
        <p className="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-400">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          Imported {outcome.result.imported_plies} moves — couldn't read move{" "}
          {outcome.result.truncated_at_ply} ({outcome.result.truncated_token}).
        </p>
      )
    case "ok":
      return (
        <p className="flex items-center gap-2 text-sm text-emerald-600 dark:text-emerald-400">
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          Imported {outcome.result.imported_plies} moves.
        </p>
      )
  }
}

const ImportPgnPopup = ({ open, onClose, onImported }: ImportPgnPopupProps) => {
  const [pgn, setPgn] = useState("")
  const [outcome, setOutcome] = useState<Outcome | null>(null)
  const [importing, setImporting] = useState(false)

  const close = () => {
    setPgn("")
    setOutcome(null)
    setImporting(false)
    onClose()
  }

  const handleImport = async () => {
    setImporting(true)
    setOutcome(null)
    try {
      const result = await importPgn(pgn.trim())
      const next = outcomeFor(result)
      setOutcome(next)
      if (next.kind !== "duplicate") onImported?.()
      if (next.kind === "ok") close()
    } catch (e) {
      setOutcome({ kind: "error", text: String(e) })
    } finally {
      setImporting(false)
    }
  }

  return (
    <Popup open={open}>
      <div className="flex w-[min(480px,92vw)] h-120 flex-col gap-4 rounded-lg border border-border bg-card p-6 shadow-xl">
        <h2 className="text-xl font-medium text-foreground/90">Import PGN</h2>
        <textarea
          value={pgn}
          onChange={(e) => setPgn(e.target.value)}
          placeholder="Paste your PGN here…"
          spellCheck={false}
          className="h-full min-h-[140px] w-full resize-y rounded-md border border-border bg-input/30 p-2 text-sm text-foreground outline-none focus:border-primary"
        />
        {outcome && <OutcomeNote outcome={outcome} />}
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={close}
            className="rounded-md px-3 py-1.5 text-sm text-foreground/70 transition-colors hover:bg-border/60"
          >
            {outcome && outcome.kind !== "error" ? "Done" : "Cancel"}
          </button>
          <button
            type="button"
            onClick={handleImport}
            disabled={!pgn.trim() || importing}
            className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-40"
          >
            {importing ? "Importing…" : "Import"}
          </button>
        </div>
      </div>
    </Popup>
  )
}

export default ImportPgnPopup
