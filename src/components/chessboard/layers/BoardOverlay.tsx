import type { ReactNode } from "react";

interface BoardOverlayProps {
  children: ReactNode;
}

// Dumb on purpose: no visibility state of its own. Callers decide whether
// to render it at all (e.g. `{isStartingGame && <BoardOverlay>...}`), and
// what it shows — this just blurs the board and centers whatever it's given.
const BoardOverlay: React.FC<BoardOverlayProps> = ({ children }) => {
  return (
    <div className="absolute inset-0 z-[60] flex items-center justify-center backdrop-blur-sm bg-black/30">
      {children}
    </div>
  );
};

export default BoardOverlay;
