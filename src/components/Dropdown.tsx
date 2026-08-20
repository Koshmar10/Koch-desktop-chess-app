import { useEffect, useRef, useState, type ReactNode } from "react";

interface DropdownProps {
  trigger: (state: { open: boolean; toggle: () => void }) => ReactNode;
  children: (close: () => void) => ReactNode;
  side?: "top" | "bottom";
  align?: "left" | "center";
}

const SIDE_CLASS: Record<NonNullable<DropdownProps["side"]>, string> = {
  top: "bottom-full mb-2",
  bottom: "top-full mt-2",
};

const ALIGN_CLASS: Record<NonNullable<DropdownProps["align"]>, string> = {
  left: "left-0",
  center: "left-1/2 -translate-x-1/2",
};

const Dropdown = ({ trigger, children, side = "bottom", align = "left" }: DropdownProps) => {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const toggle = () => setOpen((o) => !o);
  const close = () => setOpen(false);

  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };

    window.addEventListener("mousedown", handleClickOutside);
    return () => window.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  return (
    <div ref={containerRef} className="relative inline-flex items-center justify-center">
      {trigger({ open, toggle })}
      {open && (
        <div className={`absolute z-10 ${SIDE_CLASS[side]} ${ALIGN_CLASS[align]}`}>
          {children(close)}
        </div>
      )}
    </div>
  );
};

export default Dropdown;
