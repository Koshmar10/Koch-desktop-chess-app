import { Check, ChevronDown } from "lucide-react";
import Dropdown from "./Dropdown";

export interface SelectOption<T> {
  value: T;
  label: string;
}

interface SelectMenuProps<T> {
  options: SelectOption<T>[];
  value: T;
  onChange: (value: T) => void;
  // The trigger and the menu share it, so the menu lines up under the box.
  widthClass?: string;
}

/** A toolbar's pick-one menu: the choice in a box, the options under it. */
const SelectMenu = <T,>({
  options,
  value,
  onChange,
  widthClass = "w-44",
}: SelectMenuProps<T>) => {
  const selected = options.find((option) => option.value === value);
  return (
    <Dropdown
      trigger={({ toggle }) => (
        <div
          className={`${widthClass} bg-card/30 border-2 border-border rounded cursor-pointer select-none flex flex-row items-center justify-between gap-1 p-2`}
          onClick={toggle}
        >
          <span className="truncate">{selected?.label}</span>
          <ChevronDown className="shrink-0" />
        </div>
      )}
    >
      {(close) => (
        <div className="bg-card border-2 border-border rounded shadow-lg">
          {options.map((option) => (
            <div
              key={option.label}
              className={`${widthClass} px-4 py-2 hover:bg-border cursor-pointer flex flex-row justify-between`}
              onClick={() => {
                onChange(option.value);
                close();
              }}
            >
              <span className="flex-1 truncate">{option.label}</span>
              {option.value === value && <Check className="shrink-0" />}
            </div>
          ))}
        </div>
      )}
    </Dropdown>
  );
};

export default SelectMenu;
