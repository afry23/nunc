import type { ButtonHTMLAttributes, ComponentProps, ReactNode } from "react";
import { useId } from "react";

type ButtonProps = ComponentProps<"button"> & {
  variant?: "primary" | "secondary" | "ghost" | "danger";
};

const BUTTON_BASE =
  "inline-flex min-h-11 items-center justify-center gap-2 rounded-lg px-5 font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-40";

const BUTTON_VARIANTS: Record<NonNullable<ButtonProps["variant"]>, string> = {
  primary: "btn-primary tracking-widest uppercase",
  secondary:
    "bg-surface-container-high text-on-surface hover:bg-surface-bright",
  ghost:
    "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface",
  danger: "bg-surface-container-high text-red-300 hover:bg-surface-bright",
};

export function Button({
  variant = "secondary",
  className = "",
  type = "button",
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={`${BUTTON_BASE} ${BUTTON_VARIANTS[variant]} ${className}`}
      {...rest}
    />
  );
}

/** Quadratischer Icon-Button mit Pflicht-Beschriftung (Touch-Ziel ≥ 44 px). */
export function IconButton({
  label,
  className = "",
  children,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={`inline-flex size-11 items-center justify-center rounded-lg text-on-surface-variant transition-colors hover:bg-surface-bright hover:text-on-surface disabled:cursor-not-allowed disabled:opacity-30 ${className}`}
      {...rest}
    >
      {children}
    </button>
  );
}

/** Schalter mit Beschriftung (`role="switch"`). */
export function Toggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  const labelId = useId();
  const descriptionId = useId();
  return (
    <div className="flex min-h-11 items-center justify-between gap-4">
      <div>
        <p id={labelId} className="font-semibold">
          {label}
        </p>
        {description && (
          <p id={descriptionId} className="text-sm text-on-surface-variant">
            {description}
          </p>
        )}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-labelledby={labelId}
        aria-describedby={description ? descriptionId : undefined}
        onClick={() => onChange(!checked)}
        className="flex h-11 w-14 shrink-0 items-center justify-center"
      >
        <span
          className={`relative h-7 w-12 rounded-full transition-colors ${
            checked ? "bg-primary" : "bg-surface-container-highest"
          }`}
        >
          <span
            className={`absolute top-1 size-5 rounded-full bg-on-surface transition-all ${
              checked ? "left-6" : "left-1"
            }`}
          />
        </span>
      </button>
    </div>
  );
}

/** Auswahl-Chip (Filter), umschaltbar. */
export function Chip({
  selected,
  onClick,
  children,
}: {
  selected: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className={`min-h-11 rounded-full px-3 text-sm transition-colors ${
        selected
          ? "bg-secondary-fixed-dim text-on-secondary"
          : "bg-surface-container-high text-on-surface-variant hover:bg-surface-bright"
      }`}
    >
      {children}
    </button>
  );
}

/** Segmentierte Auswahl (genau ein Wert), als Radiogruppe. */
export function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
  disabled = false,
}: {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className="inline-flex gap-1 rounded-xl bg-surface-container-lowest p-1"
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          aria-checked={option.value === value}
          disabled={disabled}
          onClick={() => onChange(option.value)}
          className={`min-h-11 rounded-lg px-5 font-semibold transition-colors disabled:cursor-not-allowed ${
            option.value === value
              ? "bg-surface-container-high text-on-surface"
              : "text-on-surface-variant hover:text-on-surface"
          } ${disabled && option.value !== value ? "opacity-40" : ""}`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function SectionLabel({ children }: { children: ReactNode }) {
  return <p className="label-md text-on-surface-variant">{children}</p>;
}
