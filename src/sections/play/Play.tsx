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
} from "./GameContext";
import type { PlayerInfo } from "../../api/bindings/PlayerInfo";
import type { PieceColor } from "../../api/bindings/PieceColor";
import GameResultCard from "./GameResultCard";

const PLACEHOLDER_BLACK: PlayerInfo = { name: "Black", elo: 0 };
const PLACEHOLDER_WHITE: PlayerInfo = { name: "White", elo: 0 };

const CLOCK_TICK_MS = 250;

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
  const pieces = game ? game.state.pieces : STARTING_POSITION;
  const isFlipped = humanColor === "black";
  const isOngoing = game !== null && game.state.result === "Unfinished";

  const [now, setNow] = useState(() => Date.now());

  const liveRemainingMs = (color: PieceColor): number => {
    if (!game) return MODE_TIME_CONTROL[selectedMode].initial_ms;
    const baseline =
      color === "white"
        ? game.state.white_remaining_ms
        : game.state.black_remaining_ms;
    if (game.state.turn !== color) return baseline;
    const sinceReceived = gameReceivedAt ? now - gameReceivedAt : 0;
    return Math.max(0, baseline - game.state.elapsed_this_turn_ms - sinceReceived);
  };
  const whiteRemainingMs = liveRemainingMs("white");
  const blackRemainingMs = liveRemainingMs("black");

  // Ticks `now` once a second so the live clocks above actually count down.
  useEffect(() => {
    if (!isOngoing) return;
    const interval = setInterval(() => setNow(Date.now()), CLOCK_TICK_MS);
    return () => clearInterval(interval);
  }, [isOngoing]);

  // Ends the game the moment whoever's turn it is runs out of time.
  useEffect(() => {
    if (!isOngoing || !game) return;
    const remaining =
      game.state.turn === "white" ? whiteRemainingMs : blackRemainingMs;
    if (remaining <= 0) {
      endGame("Timeout", game.state.turn);
    }
  }, [isOngoing, game, whiteRemainingMs, blackRemainingMs, endGame]);

  const blackIsTurn = game ? game.state.turn === "black" : false;
  const whiteIsTurn = game ? game.state.turn === "white" : true;

  // Each card bundles color + playerInfo + isTurn together so they can't
  // drift out of sync with each other — the bug in the earlier draft was
  // exactly that: color and player name updated independently.
  const blackCard = {
    color: "black" as const,
    playerInfo: game ? game.black_player : PLACEHOLDER_BLACK,
    piecesTaken: getTakenPieces(pieces, "black"),
    clock: formatClock(blackRemainingMs),
    isTurn: game !== null && !isStartingGame && blackIsTurn,
    // Their turn, but they're not the human — waiting on the engine.
    isThinking: game !== null && blackIsTurn && humanColor !== "black",
  };
  const whiteCard = {
    color: "white" as const,
    playerInfo: game ? game.white_player : PLACEHOLDER_WHITE,
    piecesTaken: getTakenPieces(pieces, "white"),
    clock: formatClock(whiteRemainingMs),
    isTurn: game !== null && !isStartingGame && whiteIsTurn,
    isThinking: game !== null && whiteIsTurn && humanColor !== "white",
  };

  // Same orientation the board itself uses — the human's own card stays on
  // the bottom, closest to their controls, regardless of which color they're playing.
  const [topCard, bottomCard] = isFlipped
    ? [whiteCard, blackCard]
    : [blackCard, whiteCard];

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
            canPlayerMove={isOngoing}
            // Self-play testing: whichever color is actually to move can be
            // dragged, not just humanColor's pieces — make_move already
            // enforces "the right color for whoever's turn it is" itself,
            // so this is what lets both sides be played by hand for now.
            // Swap back to `humanColor` once Stockfish auto-replies.
            humanColor={game?.state.turn ?? null}
            onMove={makeMove}
            legalMoves={game?.state.legal_moves}
          >
            <Squares />
            <PieceLayer />
            <ArrowLayer />
            {isStartingGame && (
              <BoardOverlay>
                <LoadingSpinner message="Starting game…" />
              </BoardOverlay>
            )}
            {!isStartingGame && game !== null && !isOngoing && !isResultDismissed && (
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
