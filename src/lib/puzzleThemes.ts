// A Lichess theme tag as a person reads it: "discoveredAttack" becomes
// "Discovered attack", "mateIn2" becomes "Mate in 2".
export const formatPuzzleTheme = (theme: string): string => {
  const words = theme
    .replace(/([A-Z]|\d+)/g, " $1")
    .trim()
    .toLowerCase();
  return words.charAt(0).toUpperCase() + words.slice(1);
};
