import { useState, type ReactNode } from "react";
import { useIsWide } from "../hooks/useMediaQuery";
import { ChevronDownIcon, ChevronRightIcon } from "./icons";

/**
 * Abschnitt mit Überschrift. Breite Fenster zeigen ihn offen; schmale klappen ihn auf Wunsch auf (`defaultOpen`
 * bestimmt den Anfang). Die Überschrift bleibt eine `h2`.
 */
export function CollapsibleSection({
  title,
  defaultOpen = false,
  children,
}: {
  title: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  const wide = useIsWide();
  const [open, setOpen] = useState(defaultOpen);
  const shown = wide || open;
  return (
    <section className="rounded-2xl bg-surface-container-low p-5">
      <h2 className="font-display text-lg font-bold">
        {wide ? (
          title
        ) : (
          <button
            type="button"
            aria-expanded={open}
            onClick={() => setOpen(!open)}
            className="flex min-h-11 w-full items-center justify-between gap-2 text-left"
          >
            {title}
            {open ? <ChevronDownIcon /> : <ChevronRightIcon />}
          </button>
        )}
      </h2>
      {shown && <div className="mt-3">{children}</div>}
    </section>
  );
}
