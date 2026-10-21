import type { ReactNode } from "react";

interface StatRowProps {
  icon: ReactNode;
  label: string;
  value: string | number;
}

/**
 * One number in a side panel, with its icon and name: Play's session
 * stats, the puzzle's rating and points.
 */
const StatRow = ({ icon, label, value }: StatRowProps) => (
  <div className="flex items-center gap-2 text-md">
    {icon}
    <span className="text-foreground/50">{label}</span>
    <span className="ml-auto font-semibold text-foreground/90">{value}</span>
  </div>
);

export default StatRow;
