import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { IconButton } from "./ui";

export type MenuAction = {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
  /** Ausgeblendete Einträge zählen nicht (bequemer als Bedingungen im Aufrufer). */
  hidden?: boolean;
};

/**
 * „⋯“-Menü: selten gebrauchte Aktionen hinter einem Knopf. Escape und Klick daneben schließen, die Wahl eines
 * Eintrags ebenfalls.
 */
export function ActionMenu({
  label,
  icon,
  actions,
}: {
  /** Zugänglicher Name des Auslösers, z. B. „Aktionen: Bericht“. */
  label: string;
  icon: ReactNode;
  actions: MenuAction[];
}) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const menuId = useId();

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

  return (
    <div ref={rootRef} className="relative">
      <IconButton
        label={label}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? menuId : undefined}
        onClick={() => setOpen(!open)}
      >
        {icon}
      </IconButton>
      {open && (
        <div
          id={menuId}
          role="menu"
          aria-label={label}
          className="glass shadow-ambient absolute right-0 z-10 mt-1 flex min-w-56 max-w-[calc(100vw-2rem)] flex-col rounded-xl p-1"
        >
          {actions
            .filter((action) => !action.hidden)
            .map((action) => (
              <button
                key={action.label}
                type="button"
                role="menuitem"
                disabled={action.disabled}
                onClick={() => {
                  setOpen(false);
                  action.onSelect();
                }}
                className="min-h-11 rounded-lg px-3 text-left text-on-surface transition-colors hover:bg-surface-bright disabled:cursor-not-allowed disabled:opacity-40"
              >
                {action.label}
              </button>
            ))}
        </div>
      )}
    </div>
  );
}
