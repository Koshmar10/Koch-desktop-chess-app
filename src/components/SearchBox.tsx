import { Search } from "lucide-react";

interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}

/** A toolbar's text search: a magnifier and a borderless input. */
const SearchBox = ({ value, onChange, placeholder }: SearchBoxProps) => (
  <div className="relative flex items-center bg-input/30 border-2 border-border rounded p-2 gap-2 outline-none focus-within:border-primary transition-colors">
    <Search className="text-muted-foreground w-4 h-4 shrink-0" />
    <input
      type="text"
      placeholder={placeholder}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="bg-transparent outline-none text-foreground placeholder:text-muted-foreground w-full"
    />
  </div>
);

export default SearchBox;
