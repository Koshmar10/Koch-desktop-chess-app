import { useEffect, useState } from "react";
import { Microscope, Play as PlayIcon } from "lucide-react";
import { OutlineButton } from "../../components/buttons";
import DialogCard, { NARROW_DIALOG_PX } from "../../components/DialogCard";
import PostGameStats from "./PostGameStats";
import { useGameContext } from "./GameContext";
import type { GameScore } from "../../api/bindings/GameScore";

const ICON_SIZE_PX = 16;
const MOCK_ELO_DELTA = 18;
const ELO_COUNT_UP_MS = 1000;
const MESSAGE_CYCLE_MS = 2200;

const PERSONALITY_MESSAGES = [
  "Looking into it…",
  "Tough fight…",
  "Looking good…",
  "Oh shit…",
  "Crunching the numbers…",
  "Hang on…",
];

// The backend decides how the game went for you (`human_score`); this
// only words it.
const RESULT_MESSAGE: Record<GameScore, string> = {
  Win: "You Win",
  Draw: "Draw",
  Loss: "You Lose",
};

// Only for a game with no score — which the card never shows, since it
// waits for the game to end.
const NO_SCORE_MESSAGE = "Game over";

const useCountUp = (target: number, durationMs: number) => {
  const [value, setValue] = useState(0);

  useEffect(() => {
    let frame: number;
    const start = performance.now();

    const tick = (now: number) => {
      const progress = Math.min((now - start) / durationMs, 1);
      setValue(Math.round(target * progress));
      if (progress < 1) frame = requestAnimationFrame(tick);
    };

    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [target, durationMs]);

  return value;
};

const randomMessage = (exclude?: string): string => {
  const choices = exclude
    ? PERSONALITY_MESSAGES.filter((m) => m !== exclude)
    : PERSONALITY_MESSAGES;
  return choices[Math.floor(Math.random() * choices.length)];
};

interface ResultHeaderProps {
  message: string;
}

const ResultHeader = ({ message }: ResultHeaderProps) => {
  const isGain = MOCK_ELO_DELTA >= 0;
  const eloDelta = useCountUp(MOCK_ELO_DELTA, ELO_COUNT_UP_MS);

  return (
    <div className="flex flex-col items-center gap-1">
      <h2 className="text-2xl font-bold text-foreground/90 tracking-tight text-center">
        {message}
      </h2>
      <span
        className={`text-xl font-semibold ${isGain ? "text-primary" : "text-destructive"}`}
      >
        {isGain ? "+" : ""}
        {eloDelta} Elo
      </span>
    </div>
  );
};

const AnalyzingSpinner = () => {
  const { analysisProgress } = useGameContext();
  const [message, setMessage] = useState(() => randomMessage());

  useEffect(() => {
    const interval = setInterval(() => {
      setMessage((prev) => randomMessage(prev));
    }, MESSAGE_CYCLE_MS);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-col items-center gap-1">
      <span className="text-md font-bold text-foreground/70 text-center">
        {message}
      </span>
      {analysisProgress !== null && (
        <span className="text-sm text-foreground/50">{analysisProgress}%</span>
      )}
    </div>
  );
};

interface ResultActionsProps {
  onNewGame: () => void;
}

// Two equal choices for what's next, so neither is drawn as the primary.
const ResultActions = ({ onNewGame }: ResultActionsProps) => (
  <div className="flex items-center gap-3">
    <OutlineButton onClick={onNewGame}>
      <PlayIcon size={ICON_SIZE_PX} />
      New Game
    </OutlineButton>
    <OutlineButton>
      <Microscope size={ICON_SIZE_PX} />
      Analyze
    </OutlineButton>
  </div>
);

const GameResultCard = () => {
  const { analysis, game, humanColor, startGame, dismissResult } =
    useGameContext();

  if (!game) return null;

  // No title: the verdict below is the card's heading.
  return (
    <DialogCard onClose={dismissResult} widthPx={NARROW_DIALOG_PX}>
      <div className="flex flex-col items-center gap-4">
        <ResultHeader
          message={
            game.state.human_score
              ? RESULT_MESSAGE[game.state.human_score]
              : NO_SCORE_MESSAGE
          }
        />

        {analysis ? (
          <PostGameStats
            analysis={analysis}
            moveHistory={game.state.move_history}
            humanColor={humanColor}
          />
        ) : (
          <AnalyzingSpinner />
        )}

        <ResultActions onNewGame={startGame} />
      </div>
    </DialogCard>
  );
};

export default GameResultCard;
