import type { ReactNode } from "react";

interface PanelCardProps {
  children: ReactNode;
  // Takes the column's leftover height, for a card holding a
  // `PanelSection fill`.
  fill?: boolean;
}

const PanelCard = ({ children, fill = false }: PanelCardProps) => {
  const sizing = fill ? "min-h-0 flex-1" : "shrink-0";

  return (
    <div
      className={`flex flex-col gap-4 rounded-lg border-[1px] border-border/80 bg-secondary/40 p-3 shadow-sm ${sizing}`}
    >
      {children}
    </div>
  );
};

export default PanelCard;
