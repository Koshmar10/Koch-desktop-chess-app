interface LoadErrorProps {
  // Reads after "Couldn't load": "this game".
  what: string;
  message: string;
}

const LoadError = ({ what, message }: LoadErrorProps) => (
  <div className="max-w-sm rounded-lg border-[1px] border-destructive/50 bg-card p-4 text-sm text-foreground/80">
    Couldn't load {what}: {message}
  </div>
);

export default LoadError;
