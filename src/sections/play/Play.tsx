import { useEffect, useState } from "react";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import ArrowLayer from "../../components/chessboard/layers/ArrowLayer";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import { PlayerCard } from "../../components/chessboard/PlayerCard";
import { LoadingSpinner } from "../../components/LoadingSpinner";
import { STARTING_POSITION } from "../../components/chessboard/lib/startingPosition";
import { BOARD_PIXEL_SIZE } from "../../components/chessboard/lib/constants";
import { getTakenPieces } from "../../components/chessboard/lib/takenPieces";
import PlayControls from "./PlayControls";
import GameSidePanel from "./GameSidePanel";
import { GameProvider } from "./GameProvider";
import {
  useGameContext,
  MODE_TIME_CONTROL,
  formatClock,
  isGameOngoing,
  isTurn,
  GameMode,
} from "./GameContext";
import type { PlayerInfo } from "../../api/bindings/PlayerInfo";
import type { PieceColor } from "../../api/bindings/PieceColor";
import GameResultCard from "./GameResultCard";
import { GameCreateResponse } from "../../api/bindings/GameCreateResponse";

const PLACEHOLDER_BLACK: PlayerInfo = { name: "Black", elo: 0 };
const PLACEHOLDER_WHITE: PlayerInfo = { name: "White", elo: 0 };

const CLOCK_TICK_MS = 250;

const liveRemainingMs = (
  game: GameCreateResponse | null,
  gameReceivedAt: number | null,
  color: PieceColor,
  selectedMode: GameMode,
  now: number,
): number => {
  if (!game) return MODE_TIME_CONTROL[selectedMode].initial_ms;

  const baseline =
    color === "white"
      ? game.state.white_remaining_ms
      : game.state.black_remaining_ms;
  if (game.state.turn !== color) return baseline;

  const sinceReceived = gameReceivedAt ? now - gameReceivedAt : 0;
  return Math.max(0, baseline - game.state.elapsed_this_turn_ms - sinceReceived);
};

// Reads GameContext, so it has to be a descendant of <GameProvider>, not the
// same component that renders the provider.
const PlayBoard = () => {
  const {
    game,
    gameReceivedAt,
    isResultDismissed,
    humanColor,
    selectedMode,
    isStartingGame,
    makeMove,
    endGame,
  } = useGameContext();
  const [now, setNow] = useState(() => Date.now());

  const pieces = game ? game.state.pieces : STARTING_POSITION;
  const isFlipped = humanColor === "black";
  const hasGame = game !== null;
  const isOngoing = isGameOngoing(game);
  const isBlackTurn = isTurn(game, "black");
  const isWhiteTurn = isTurn(game, "white");
  const canShowTurnIndicator = hasGame && !isStartingGame;
  const showResultCard = !isStartingGame && hasGame && !isOngoing && !isResultDismissed;
  // Now that `humanColor` is fixed rather than tracking whoever's turn it
  // is (self-play testing is over — the engine replies on its own), this
  // has to check whose turn it actually is, not just whether a game exists.
  const canPlayerMove = isOngoing && humanColor !== null && isTurn(game, humanColor);
  const whiteRemainingMs = liveRemainingMs(game, gameReceivedAt, "white", selectedMode, now);
  const blackRemainingMs = liveRemainingMs(game, gameReceivedAt, "black", selectedMode, now);

  const blackCard = {
    color: "black" as const,
    playerInfo: game ? game.black_player : PLACEHOLDER_BLACK,
    piecesTaken: getTakenPieces(pieces, "black"),
    clock: formatClock(blackRemainingMs),
    isTurn: canShowTurnIndicator && isBlackTurn,
    // Their turn, but they're not the human — waiting on the engine.
    isThinking: hasGame && isBlackTurn && humanColor !== "black",
  };
  const whiteCard = {
    color: "white" as const,
    playerInfo: game ? game.white_player : PLACEHOLDER_WHITE,
    piecesTaken: getTakenPieces(pieces, "white"),
    clock: formatClock(whiteRemainingMs),
    isTurn: canShowTurnIndicator && isWhiteTurn,
    isThinking: hasGame && isWhiteTurn && humanColor !== "white",
  };

  const [topCard, bottomCard] = isFlipped
    ? [whiteCard, blackCard]
    : [blackCard, whiteCard];

  useEffect(() => {
    if (!isOngoing) return;
    const interval = setInterval(() => setNow(Date.now()), CLOCK_TICK_MS);
    return () => clearInterval(interval);
  }, [isOngoing]);

  useEffect(() => {
    if (!isOngoing || !game) return;
    const remaining =
      game.state.turn === "white" ? whiteRemainingMs : blackRemainingMs;
    if (remaining <= 0) {
      endGame("Timeout", game.state.turn);
    }
  }, [isOngoing, game, whiteRemainingMs, blackRemainingMs, endGame]);

  return (
    <div className="flex flex-row justify-center items-center gap-4 w-full h-full">
      <div className="flex flex-row gap-4">
        <div
          className="flex flex-col justify-center gap-2 w-full"
          style={{ maxWidth: BOARD_PIXEL_SIZE }}
        >
          <PlayerCard display={true} {...topCard} />
          <Chessboard
            pieces={pieces}
            flipped={isFlipped}
            canPlayerMove={canPlayerMove}
            humanColor={humanColor}
            onMove={makeMove}
            legalMoves={game?.state.legal_moves}
            lastMove={game?.state.last_move}
          >
            <Squares />
            <PieceLayer />
            <ArrowLayer />
            {isStartingGame && (
              <BoardOverlay>
                <LoadingSpinner message="Starting game…" />
              </BoardOverlay>
            )}
            {showResultCard && (
              <BoardOverlay>
                <GameResultCard />
              </BoardOverlay>
            )}
          </Chessboard>
          <PlayerCard display={true} {...bottomCard} />
          <PlayControls />
        </div>
        <GameSidePanel />
      </div>
    </div>
  );
};

const Play = () => {
  return (
    <GameProvider>
      <PlayBoard />
    </GameProvider>
  );
};

export default Play;
