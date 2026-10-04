import { useEffect } from "react";
import { useT } from "../i18n";
import { useUiStore } from "../stores/uiStore";
import { CloseIcon } from "./icons";
import { IconButton } from "./ui";

/** Einblendung für Fehler und Hinweise; Fehler bleiben, bis sie geschlossen werden. */
export function Toast() {
  const t = useT();
  const notice = useUiStore((s) => s.notice);
  const dismiss = useUiStore((s) => s.dismiss);

  useEffect(() => {
    if (!notice || notice.kind === "error") return;
    const timer = setTimeout(dismiss, 4000);
    return () => clearTimeout(timer);
  }, [notice, dismiss]);

  if (!notice) return null;
  return (
    <div
      role={notice.kind === "error" ? "alert" : "status"}
      className="glass shadow-ambient fixed right-4 bottom-24 z-50 flex max-w-md items-center gap-3 rounded-xl px-4 py-2 md:bottom-6"
    >
      <p
        className={notice.kind === "error" ? "text-red-300" : "text-on-surface"}
      >
        {notice.message}
      </p>
      <IconButton label={t("toast.close")} onClick={dismiss}>
        <CloseIcon />
      </IconButton>
    </div>
  );
}
