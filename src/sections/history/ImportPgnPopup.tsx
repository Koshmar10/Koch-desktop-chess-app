import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import Popup from "../../components/popup/Popup"

interface ImportPgnPopupProps {
  open: boolean
  onClose: () => void
  // Fired after a successful import, for the caller to refresh its list.
  onImported?: () => void
}

const ImportPgnPopup = ({ open, onClose, onImported }: ImportPgnPopupProps) => {
  const [pgn, setPgn] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [importing, setImporting] = useState(false)

  const close = () => {
    setPgn("")
    setError(null)
    setImporting(false)
    onClose()
  }

  const handleImport = async () => {
    setImporting(true)
    setError(null)
    try {
      // TODO: the backend `import_pgn` command doesn't exist yet — this
      // will fail until it's added.
      await invoke("import_pgn", { pgn: pgn.trim() })
      onImported?.()
      close()
    } catch (e) {
      setError(String(e))
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
        {error && (
          <p className="text-sm text-red-500 dark:text-red-400">{error}</p>
        )}
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={close}
            className="rounded-md px-3 py-1.5 text-sm text-foreground/70 transition-colors hover:bg-border/60"
          >
            Cancel
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
