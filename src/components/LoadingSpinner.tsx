import { Loader2 } from "lucide-react";

interface LoadingSpinnerProps {
  message: string;
}

export const LoadingSpinner = ({ message }: LoadingSpinnerProps) => (
  <div className="size-[22%] flex flex-col items-center justify-center py-4 gap-3 bg-card text-card-foreground rounded-lg border border-border shadow-md">
    <Loader2 className="animate-spin text-primary" size={50} />
    <span className="text-md p-2 font-bold text-card-foreground/70 text-center">
      {message}
    </span>
  </div>
);
