import { useEffect, useState } from "react"
import { getGames, deleteGame, analyzeGame } from "../../api/game"
import { GameSummary } from "../../api/bindings/GameSummary"
import { PieceColor } from "../../api/bindings/PieceColor"
import GameCard from "./GameCard"
import AnalyzeSidePopup from "./AnalyzeSidePopup"

const MissingGames = () => (
  <div className="text-sm text-foreground/50 px-6 py-8 text-center">
    No existing games to retrieve
  </div>
)

const GameCardsSection = ({ reloadKey }: { reloadKey?: number }) => {
  const [gameData, setGameData] = useState<GameSummary[] | null>(null)
  const [sideNeeded, setSideNeeded] = useState<GameSummary | null>(null)
  useEffect(() => {
    getGames().then(setGameData).catch(console.error)
  }, [reloadKey])

  const handleDelete = (game: GameSummary) => {
    deleteGame(game.game_id)
      .then(() =>
        setGameData(
          (prev) => prev?.filter((g) => g.game_id !== game.game_id) ?? null,
        ),
      )
      .catch(console.error)
  }

  // Fire-and-forget — the analysis runs on the backend queue. A koch game
  // is graded from the side you played; an imported game has no inherent
  // side, so you pick one every time (defaulting to the last pick).
  const handleAnalyze = (game: GameSummary) => {
    if (game.source !== "koch") {
      setSideNeeded(game)
      return
    }
    analyzeGame(game.game_id).catch(console.error)
  }

  const handlePickSide = (color: PieceColor) => {
    if (sideNeeded) analyzeGame(sideNeeded.game_id, color).catch(console.error)
    setSideNeeded(null)
  }

  return (
    <>
      {gameData && gameData.length > 0 ? (
        <div className="flex flex-wrap gap-3 px-6 py-4">
          {gameData.map((game) => (
            <div key={game.game_id} className="w-full max-w-[calc(25%-0.6875rem)]">
              <GameCard
                gameSummary={game}
                onAnalyze={handleAnalyze}
                onDelete={handleDelete}
              />
            </div>
          ))}
        </div>
      ) : (
        <MissingGames />
      )}
      <AnalyzeSidePopup
        game={sideNeeded}
        onPick={handlePickSide}
        onClose={() => setSideNeeded(null)}
      />
    </>
  )
}

export default GameCardsSection
