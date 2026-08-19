import { useState } from 'react';

import { Check, ChevronDown, RefreshCcw, Search, Upload } from 'lucide-react';
import { TooltipButton } from '../../components/TooltipButton';

type SelectOption = "Last Month" | "Last 3 Months" | "All Time";
const SelectOptions: SelectOption[] = ["Last Month", "Last 3 Months", "All Time"]

const SearchBox = () => (
  <div className="relative flex items-center bg-input/30 border-2 border-border rounded p-2 gap-2 outline-none focus-within:border-primary transition-colors">
    <Search className="text-muted-foreground w-4 h-4" />
    <input
      type="text"
      placeholder="Search games..."
      className="bg-transparent outline-none text-foreground placeholder:text-muted-foreground w-full"
    />
  </div>
)

const MonthSelectMenu = ({
  selectOption,
  onSelect,
}: {
  selectOption: SelectOption;
  onSelect: (option: SelectOption) => void;
}) => (
  <div className="absolute left-0 top-full mt-2 bg-card border-2 border-border rounded shadow-lg">
    {SelectOptions.map(option => (
      <div
        key={option}
        className="px-4 py-2 hover:bg-border cursor-pointer flex flex-row w-44 justify-between"
        onClick={() => onSelect(option)}
      >
        <span className="flex-1">{option}</span>
        <span>{option === selectOption ? <Check /> : ''}</span>
      </div>
    ))}
  </div>
)

const MonthSelect = () => {
  const [open, setOpen] = useState(false);
  const [selectOption, setSelectOption] = useState<SelectOption>("Last Month");

  return (
    <div
      className="w-44 relative bg-card/30 border-2 border-border rounded cursor-pointer select-none flex flex-row items-center justify-between p-2"
      onClick={() => setOpen((o) => !o)}
    >
      <span className="block">{selectOption}</span>
      <ChevronDown />
      {open && (
        <MonthSelectMenu selectOption={selectOption} onSelect={setSelectOption} />
      )}
    </div>
  )
}

const HistoryToolbar = () => {
  return (
    <div className="gap-4 text-foreground border-b-2 border-border flex flex-row items-center p-3">
      <SearchBox />
      <MonthSelect />
      <TooltipButton icon={<RefreshCcw />} tooltip={"Sync with chess.com"} />
      <TooltipButton icon={<Upload />} tooltip={"Import PGN"} />
    </div>
  );
};

export default HistoryToolbar;
