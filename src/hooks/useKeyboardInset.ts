import { useEffect, useState } from "react";

/** Ab dieser Höhe (px) gilt die Bildschirmtastatur als geöffnet (Browserleisten bleiben darunter). */
const KEYBOARD_MIN_HEIGHT = 120;

const isTextField = (element: Element | null): boolean =>
  element instanceof HTMLInputElement ||
  element instanceof HTMLTextAreaElement ||
  element instanceof HTMLSelectElement;

/**
 * Erkennt die Bildschirmtastatur über den Visual Viewport (iOS überlagert die Seite, ohne sie zu verkleinern),
 * setzt `--keyboard-inset` und holt das fokussierte Eingabefeld in den sichtbaren Bereich.
 * Liefert `true`, solange die Tastatur offen ist (dann verschwindet die Tab-Leiste).
 */
export function useKeyboardInset(): boolean {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const viewport = window.visualViewport;
    if (!viewport) return;

    const update = (): void => {
      const inset = Math.max(
        0,
        window.innerHeight -
          viewport.height * viewport.scale -
          viewport.offsetTop,
      );
      const isOpen = inset >= KEYBOARD_MIN_HEIGHT;
      document.documentElement.style.setProperty(
        "--keyboard-inset",
        `${isOpen ? inset : 0}px`,
      );
      setOpen(isOpen);
      if (isOpen && isTextField(document.activeElement))
        document.activeElement?.scrollIntoView({ block: "center" });
    };
    const onFocus = (event: FocusEvent): void => {
      // die Tastatur fährt erst nach dem Fokus aus; `resize` des Viewports holt das Feld dann nach
      if (
        isTextField(event.target as Element) &&
        viewport.height < window.innerHeight
      )
        (event.target as Element).scrollIntoView({ block: "center" });
    };

    viewport.addEventListener("resize", update);
    document.addEventListener("focusin", onFocus);
    update();
    return () => {
      viewport.removeEventListener("resize", update);
      document.removeEventListener("focusin", onFocus);
      document.documentElement.style.removeProperty("--keyboard-inset");
    };
  }, []);

  return open;
}
