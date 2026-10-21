interface LoadErrorProps {
  // What didn't load, as it reads after "Couldn't load": "this game".
  what: string;
  message: string;
}

/**
 * Shown over the board when what it should show didn't load: the reason
 * worded as the backend sent it, after a line saying what failed.
 */
const LoadError = ({ what, message }: LoadErrorProps) => (
  <div className="max-w-sm rounded-lg border-[1px] border-destructive/50 bg-card p-4 text-sm text-foreground/80">
    Couldn't load {what}: {message}
  </div>
);

export default LoadError;
