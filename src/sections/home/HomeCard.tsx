import { LucideIcon } from "lucide-react";

interface HomeCardProps {
  icon: LucideIcon;
  value: string | number;
  label: string;
  delta?: string;
  className?: string;
}

const HomeCard = ({
  icon: Icon,
  value,
  label,
  delta,
  className = "",
}: HomeCardProps) => {
  return (
    <div
      className={`rounded-lg shadow-sm p-4 flex flex-col justify-start bg-card/40 hover:bg-card/60 text-foreground ${className}`}
      data-testid="home-card"
    >
      <div className="mb-3 p-3 rounded-xl bg-primary/15 w-fit">
        <Icon size={20} className="h-5 w-5 text-primary" strokeWidth={1.5} />
      </div>
      <div>
        <div className="text-2xl font-bold">{value}</div>
        <div className="text-xs text-secondary/70">{label}</div>
        {delta && (
          <div className="text-[10px] text-green-500 mt-1">{delta}</div>
        )}
      </div>
    </div>
  );
};

export default HomeCard;
