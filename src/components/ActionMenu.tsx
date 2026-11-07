import {
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
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
  const [pos, setPos] = useState<{ top: number; right: number } | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuId = useId();

  /** Das Menü hängt am Body (Position fixed), damit `overflow-hidden` der Karte es nicht abschneidet. */
  const toggle = (): void => {
    if (!open && rootRef.current) {
      const rect = rootRef.current.getBoundingClientRect();
      setPos({
        top: rect.bottom + 4,
        right: Math.max(8, window.innerWidth - rect.right),
      });
    }
    setOpen(!open);
  };

  /**
   * Wählt die Seite mit mehr Platz (unter oder über dem Auslöser) und begrenzt die Höhe auf den Rest des Fensters;
   * was nicht passt, scrollt im Menü.
   */
  useLayoutEffect(() => {
    const el = menuRef.current;
    if (!open || !el || !rootRef.current) return;
    const margin = 8;
    const trigger = rootRef.current.getBoundingClientRect();
    const below = window.innerHeight - trigger.bottom - 4 - margin;
    const above = trigger.top - 4 - margin;
    const full = el.scrollHeight;
    const placeAbove = full > below && above > below;
    const room = Math.max(0, placeAbove ? above : below);
    const height = Math.min(full, room);
    el.style.maxHeight = `${room}px`;
    el.style.top = `${placeAbove ? trigger.top - 4 - height : trigger.bottom + 4}px`;
  }, [open, pos]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (
        !rootRef.current?.contains(target) &&
        !menuRef.current?.contains(target)
      )
        setOpen(false);
    };
    const close = (): void => setOpen(false);
    /** Scrollen im Menü selbst darf es nicht schließen. */
    const closeOnPageScroll = (event: Event): void => {
      if (!menuRef.current?.contains(event.target as Node)) close();
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", close);
    window.addEventListener("scroll", closeOnPageScroll, true);
    return () => {
      window.removeEventListener("resize", close);
      window.removeEventListener("scroll", closeOnPageScroll, true);
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
        onClick={toggle}
      >
        {icon}
      </IconButton>
      {open &&
        pos &&
        createPortal(
          <div
            ref={menuRef}
            style={{ top: pos.top, right: pos.right }}
            id={menuId}
            role="menu"
            aria-label={label}
            className="glass shadow-ambient fixed z-50 flex min-w-56 overflow-y-auto overscroll-contain max-w-[calc(100vw-2rem)] flex-col rounded-xl p-1"
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
          </div>,
          document.body,
        )}
    </div>
  );
}
