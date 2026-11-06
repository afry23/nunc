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

  /** Passt das Menü unten nicht mehr ins Fenster, klappt es nach oben auf. */
  useLayoutEffect(() => {
    if (!open || !menuRef.current || !rootRef.current) return;
    const menu = menuRef.current.getBoundingClientRect();
    const trigger = rootRef.current.getBoundingClientRect();
    if (
      menu.bottom > window.innerHeight - 8 &&
      trigger.top - 4 - menu.height >= 8
    ) {
      menuRef.current.style.top = `${trigger.top - 4 - menu.height}px`;
    }
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
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", close);
    window.addEventListener("scroll", close, true);
    return () => {
      window.removeEventListener("resize", close);
      window.removeEventListener("scroll", close, true);
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
            className="glass shadow-ambient fixed z-50 flex min-w-56 max-w-[calc(100vw-2rem)] flex-col rounded-xl p-1"
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
