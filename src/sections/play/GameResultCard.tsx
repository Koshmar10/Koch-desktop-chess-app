import { useEffect, useState, type ReactNode } from "react";
import { Microscope, Play as PlayIcon, X } from "lucide-react";
import PostGameStats from "./PostGameStats";
import { useGameContext } from "./GameContext";
import type { GameResult } from "../../api/bindings/GameResult";
import type { PieceColor } from "../../api/bindings/PieceColor";

const ICON_SIZE = 16;
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

const getResultMessage = (
  result: GameResult,
  humanColor: PieceColor | null,
): string => {
  if (result === "Draw") return "Draw";
  const humanWon =
    (result === "WhiteWin" && humanColor === "white") ||
    (result === "BlackWin" && humanColor === "black");
  return humanWon ? "You Win" : "You Lose";
};

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

interface CloseButtonProps {
  onClick: () => void;
}

const CloseButton = ({ onClick }: CloseButtonProps) => (
  <button
    type="button"
    aria-label="Close"
    onClick={onClick}
    className="absolute top-3 right-3 p-1 rounded-md text-foreground/40 hover:text-foreground/80 hover:bg-primary/10 transition-colors"
  >
    <X size={18} />
  </button>
);

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

interface ActionButtonProps {
  icon: ReactNode;
  label: string;
  onClick?: () => void;
}

const ActionButton = ({ icon, label, onClick }: ActionButtonProps) => (
  <button
    type="button"
    onClick={onClick}
    className="flex items-center justify-center gap-2 w-34 px-2 py-2 rounded-md border-[1px] border-primary/40 text-foreground/90 text-md hover:bg-primary/10 transition-colors"
  >
    {icon}
    {label}
  </button>
);

interface ResultActionsProps {
  onNewGame: () => void;
}

const ResultActions = ({ onNewGame }: ResultActionsProps) => (
  <div className="flex items-center gap-3 mt-auto">
    <ActionButton icon={<PlayIcon size={ICON_SIZE} />} label="New Game" onClick={onNewGame} />
    <ActionButton icon={<Microscope size={ICON_SIZE} />} label="Analyze" />
  </div>
);

const GameResultCard = () => {
  const { analysis, game, humanColor, startGame, dismissResult } = useGameContext();

  if (!game) return null;

  return (
    <div className="relative h-[80%] flex flex-col items-center gap-4 py-6 px-8 rounded-xl bg-card border-[1px] border-border/80 shadow-lg">
      <CloseButton onClick={dismissResult} />

      <ResultHeader message={getResultMessage(game.state.result, humanColor)} />

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
  );
};

export default GameResultCard;
