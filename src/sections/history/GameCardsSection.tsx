import { useEffect, useState } from "react"
import { getGames } from "../../api/game"
import { GameSummary } from "../../api/bindings/GameSummary"
import GameCard from "./GameCard"

const MissingGames = () => (
  <div className="text-sm text-foreground/50 px-6 py-8 text-center">
    No existing games to retrieve
  </div>
)

const GameCardsSection = () => {
  const [gameData, setGameData] = useState<GameSummary[] | null>(null)
  useEffect(() => {
    getGames().then(setGameData).catch(console.error)
  }, [])
  return (
    <>
      {gameData && gameData.length > 0 ? (
        <div className="flex flex-wrap gap-3 px-6 py-4">
          {gameData.map((game) => (
            <div key={game.game_id} className="w-full max-w-[calc(25%-0.6875rem)]">
              <GameCard gameSummary={game} />
            </div>
          ))}
        </div>
      ) : (
        <MissingGames />
      )}
    </>
  )
}

export default GameCardsSection