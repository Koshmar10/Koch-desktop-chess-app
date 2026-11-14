import type { MomentKind } from "../../api/bindings/MomentKind";
import type { MomentOutcome } from "../../api/bindings/MomentOutcome";
import type { MomentSort } from "../../api/bindings/MomentSort";
import type { Phase } from "../../api/bindings/Phase";
import type { PieceType } from "../../api/bindings/PieceType";
import { RotateCcw } from "lucide-react";
import SearchBox from "../../components/SearchBox";
import SelectMenu, { type SelectOption } from "../../components/SelectMenu";
import TooltipButton from "../../components/TooltipButton";
import { MOMENT_KIND_LABEL, MOMENT_OUTCOME_LABEL } from "../../lib/moments";
import { PERIODS, type Period } from "../../lib/period";
import {
  ALL_OUTCOMES,
  DEFAULT_MOMENT_FILTERS,
  KIND_OUTCOMES,
  sameFilters,
  type MomentFilters,
} from "./filters";

const SELECT_WIDTH_CLASS = "w-40";

const HANDLED_OPTIONS: SelectOption<boolean | null>[] = [
  { value: false, label: "Not handled well" },
  { value: true, label: "Handled well" },
  { value: null, label: "All outcomes" },
];

const KIND_OPTIONS: SelectOption<MomentKind | null>[] = [
  { value: null, label: "All kinds" },
  ...(Object.keys(MOMENT_KIND_LABEL) as MomentKind[]).map((kind) => ({
    value: kind,
    label: MOMENT_KIND_LABEL[kind],
  })),
];

const PHASE_OPTIONS: SelectOption<Phase | null>[] = [
  { value: null, label: "All phases" },
  { value: "opening", label: "Opening" },
  { value: "middlegame", label: "Middlegame" },
  { value: "endgame", label: "Endgame" },
];

const PIECE_OPTIONS: SelectOption<PieceType | null>[] = [
  { value: null, label: "All pieces" },
  { value: "queen", label: "Queen" },
  { value: "rook", label: "Rook" },
  { value: "bishop", label: "Bishop" },
  { value: "knight", label: "Knight" },
  { value: "pawn", label: "Pawn" },
];

const PERIOD_OPTIONS: SelectOption<Period>[] = PERIODS.map((period) => ({
  value: period,
  label: period,
}));

const SORT_OPTIONS: SelectOption<MomentSort>[] = [
  { value: "costliest", label: "Costliest first" },
  { value: "most_recent", label: "Most recent" },
];

const outcomeOptions = (
  kind: MomentKind | null,
): SelectOption<MomentOutcome | null>[] => [
  { value: null, label: "Any result" },
  ...(kind === null ? ALL_OUTCOMES : KIND_OUTCOMES[kind]).map((outcome) => ({
    value: outcome,
    label: MOMENT_OUTCOME_LABEL[outcome],
  })),
];

interface MomentFilterBarProps {
  filters: MomentFilters;
  onChange: (filters: MomentFilters) => void;
  search: string;
  onSearchChange: (search: string) => void;
}

const MomentFilterBar = ({
  filters,
  onChange,
  search,
  onSearchChange,
}: MomentFilterBarProps) => {
  const set = <K extends keyof MomentFilters>(
    key: K,
    value: MomentFilters[K],
  ) => onChange({ ...filters, [key]: value });

  // A kind that can't end in the chosen outcome would list nothing, so
  // the outcome goes back to any.
  const setKind = (kind: MomentKind | null) => {
    const fits =
      kind === null ||
      filters.outcome === null ||
      KIND_OUTCOMES[kind].includes(filters.outcome);
    onChange({ ...filters, kind, outcome: fits ? filters.outcome : null });
  };

  const isDefault =
    sameFilters(filters, DEFAULT_MOMENT_FILTERS) && search === "";

  return (
    <div className="flex flex-row flex-wrap items-center gap-3 border-b-2 border-border p-3 text-foreground">
      <SearchBox
        value={search}
        onChange={onSearchChange}
        placeholder="Opponent or opening..."
      />
      <SelectMenu
        options={HANDLED_OPTIONS}
        value={filters.handled}
        onChange={(handled) => set("handled", handled)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={KIND_OPTIONS}
        value={filters.kind}
        onChange={setKind}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={outcomeOptions(filters.kind)}
        value={filters.outcome}
        onChange={(outcome) => set("outcome", outcome)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={PHASE_OPTIONS}
        value={filters.phase}
        onChange={(phase) => set("phase", phase)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={PIECE_OPTIONS}
        value={filters.piece}
        onChange={(piece) => set("piece", piece)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={PERIOD_OPTIONS}
        value={filters.period}
        onChange={(period) => set("period", period)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <SelectMenu
        options={SORT_OPTIONS}
        value={filters.sort}
        onChange={(sort) => set("sort", sort)}
        widthClass={SELECT_WIDTH_CLASS}
      />
      <TooltipButton
        icon={RotateCcw}
        tooltip="Reset filters"
        onClick={() => {
          onChange(DEFAULT_MOMENT_FILTERS);
          onSearchChange("");
        }}
        disabled={isDefault}
      />
    </div>
  );
};

export default MomentFilterBar;
