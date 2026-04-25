import { ButtonHTMLAttributes, ReactNode } from "react";

type BtnVariant = "neutral" | "ghost" | "danger";

interface BtnProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: BtnVariant;
  kbd?: string;
  icon?: ReactNode;
}

export function Btn({
  variant = "neutral",
  kbd,
  icon,
  children,
  className = "",
  ...rest
}: BtnProps) {
  const base =
    "h-8 px-3 rounded-[6px] inline-flex items-center justify-center gap-2 text-[12px] tight font-medium focus-ring transition-colors disabled:opacity-50 disabled:cursor-not-allowed";
  const styles: Record<BtnVariant, string> = {
    neutral: "fg-0 hover:bg-[var(--ink-3)]",
    ghost: "fg-2 hover:fg-0 hover:bg-[var(--ink-3)]",
    danger: "fg-0 hover:bg-[oklch(0.30_0.10_25)]",
  };
  const border: Record<BtnVariant, string> = {
    neutral: "border border-[var(--hair-2)] surface-3",
    ghost: "border border-transparent",
    danger: "border border-[oklch(0.45_0.18_25)] bg-[oklch(0.22_0.10_25)]",
  };
  return (
    <button
      {...rest}
      className={`${base} ${styles[variant]} ${border[variant]} ${className}`}
    >
      {icon && <span className="fg-3">{icon}</span>}
      <span>{children}</span>
      {kbd && <span className="kbd ml-1">{kbd}</span>}
    </button>
  );
}

interface IconBtnProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
}

export function IconBtn({
  active,
  children,
  className = "",
  ...rest
}: IconBtnProps) {
  return (
    <button
      {...rest}
      className={`w-7 h-7 rounded-[6px] inline-flex items-center justify-center transition-colors focus-ring ${
        active
          ? "surface-3 fg-0"
          : "fg-2 hover:fg-0 hover:bg-[var(--ink-3)]"
      } ${className}`}
    >
      {children}
    </button>
  );
}

interface ChipProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
  icon?: ReactNode;
}

export function Chip({
  active,
  icon,
  children,
  className = "",
  ...rest
}: ChipProps) {
  return (
    <button
      {...rest}
      className={`h-7 px-2.5 rounded-[6px] inline-flex items-center gap-1.5 text-[12px] tight transition-colors focus-ring ${
        active
          ? "bg-accent-soft accent border border-[var(--accent-soft)]"
          : "fg-2 hover:fg-0 border border-transparent hover:bg-[var(--ink-3)]"
      } ${className}`}
    >
      {icon && <span className={active ? "" : "fg-4"}>{icon}</span>}
      <span>{children}</span>
    </button>
  );
}

export function Kicker({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <div className={`wide-kicker ${className}`}>{children}</div>;
}

export function Kbd({ children }: { children: ReactNode }) {
  return <span className="kbd">{children}</span>;
}

interface SectionProps {
  kicker?: ReactNode;
  action?: ReactNode;
  children: ReactNode;
}

export function Section({ kicker, action, children }: SectionProps) {
  return (
    <div className="mt-5">
      {(kicker || action) && (
        <div className="mb-1.5 px-2 flex items-center justify-between">
          {kicker && <Kicker>{kicker}</Kicker>}
          {action}
        </div>
      )}
      <div className="space-y-[1px]">{children}</div>
    </div>
  );
}

interface SheetProps {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  width?: number;
  children: ReactNode;
}

/**
 * Sheet — modal-ish overlay used by Preferences, BulkTagSheet, BackupSheet.
 * Closes on Esc and backdrop click ONLY for non-destructive sheets;
 * confirm dialogs should use a stricter wrapper.
 */
export function Sheet({ open, onClose, title, width = 560, children }: SheetProps) {
  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-40 flex items-start justify-center pt-[12vh]"
      style={{ background: "oklch(0 0 0 / 0.45)", backdropFilter: "blur(4px)" }}
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
    >
      <div
        className="rounded-[10px] overflow-hidden rise surface-2"
        style={{
          width,
          border: "1px solid var(--hair-2)",
          boxShadow: "0 40px 80px -20px oklch(0 0 0 / 0.6)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {title && (
          <div className="h-12 flex items-center px-4 border-b hair">
            <div className="text-[14px] tight fg-0 font-medium flex-1">{title}</div>
            <Kbd>ESC</Kbd>
          </div>
        )}
        {children}
      </div>
    </div>
  );
}
