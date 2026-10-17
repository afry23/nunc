import { useEffect, useId, useRef, useState } from "react";
import { ChevronDownIcon } from "./icons";
import { Button, Chip } from "./ui";

/** Aufklappbare Mehrfachauswahl für eine Filtergruppe (Prioritäten, Projekte, Kontexte). */
export function FilterMenu({
  label,
  options,
  selected,
  onToggle,
}: {
  label: string;
  options: string[];
  selected: string[];
  onToggle: (value: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const panelId = useId();

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent): void => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  if (options.length === 0) return null;

  return (
    <div ref={rootRef} className="relative">
      <Button
        aria-expanded={open}
        aria-controls={open ? panelId : undefined}
        onClick={() => setOpen(!open)}
        className="px-3 text-sm"
      >
        {label}
        {selected.length > 0 && (
          <span
            aria-hidden="true"
            className="rounded-full bg-primary px-2 text-xs text-on-primary tabular-nums"
          >
            {selected.length}
          </span>
        )}
        <ChevronDownIcon size={16} />
      </Button>
      {open && (
        <div
          id={panelId}
          role="group"
          aria-label={label}
          className="glass shadow-ambient absolute left-0 z-10 mt-2 flex max-h-64 w-72 max-w-[calc(100vw-3rem)] flex-wrap gap-2 overflow-y-auto rounded-xl p-3"
        >
          {options.map((option) => (
            <Chip
              key={option}
              selected={selected.includes(option)}
              onClick={() => onToggle(option)}
            >
              {option}
            </Chip>
          ))}
        </div>
      )}
    </div>
  );
}
