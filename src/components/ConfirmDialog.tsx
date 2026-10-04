import { useEffect, useId, useRef, type ReactNode } from "react";
import { useT } from "../i18n";
import { Button } from "./ui";

/** Bestätigungsdialog (modal). Escape und „Abbrechen“ schließen ohne Wirkung; der Fokus liegt zuerst auf „Abbrechen“. */
export function ConfirmDialog({
  title,
  confirmLabel,
  danger = false,
  busy = false,
  onConfirm,
  onCancel,
  children,
}: {
  title: string;
  confirmLabel: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  children: ReactNode;
}) {
  const t = useT();
  const titleId = useId();
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            onCancel();
          }
        }}
        className="glass shadow-ambient w-full max-w-md rounded-2xl p-6"
      >
        <h2 id={titleId} className="headline-md">
          {title}
        </h2>
        <div className="mt-3 flex flex-col gap-2 text-on-surface-variant">
          {children}
        </div>
        <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <Button ref={cancel} variant="ghost" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
          <Button
            variant={danger ? "danger" : "primary"}
            disabled={busy}
            onClick={onConfirm}
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
