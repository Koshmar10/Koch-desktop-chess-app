import type { ComponentType, ReactNode } from "react";
import AnalyzerProvider from "../sections/analyzer/AnalyzerProvider";
import GameProvider from "../sections/play/GameProvider";
import PuzzleProvider from "../sections/puzzle/PuzzleProvider";
import AppProvider from "./AppProvider";

// Outermost first: each may use the contexts listed before it. The screens'
// providers sit above the routes so their state outlives the screen, which
// GameProvider needs to keep receiving engine and analysis events.
const PROVIDERS: ComponentType<{ children: ReactNode }>[] = [
  AppProvider,
  AnalyzerProvider,
  GameProvider,
  PuzzleProvider,
];

interface ProvidersProps {
  children: ReactNode;
}

const Providers = ({ children }: ProvidersProps) =>
  PROVIDERS.reduceRight<ReactNode>(
    (inner, Provider) => <Provider>{inner}</Provider>,
    children,
  );

export default Providers;
