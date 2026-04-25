import { CSSProperties, MouseEvent, ReactNode } from "react";

type Variant = "primary" | "neutral" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

interface Props {
  children?: ReactNode;
  onClick?: (e: MouseEvent<HTMLButtonElement>) => void;
  variant?: Variant;
  size?: Size;
  icon?: ReactNode;
  kbd?: string;
  disabled?: boolean;
  fullWidth?: boolean;
  title?: string;
  type?: "button" | "submit";
}

export default function Btn({
  children,
  onClick,
  variant = "neutral",
  size = "md",
  icon,
  kbd,
  disabled = false,
  fullWidth = false,
  title,
  type = "button",
}: Props) {
  const sizeCls =
    size === "sm"
      ? "h-7 px-2.5 text-[11.5px]"
      : size === "lg"
      ? "h-10 px-4 text-[13.5px]"
      : "h-8 px-3 text-[12.5px]";

  const styles: CSSProperties =
    variant === "primary"
      ? {
          background: "var(--accent)",
          color: "oklch(0.16 0.006 60)",
          border: "1px solid var(--accent-dim)",
        }
      : variant === "danger"
      ? {
          background: "var(--err-soft)",
          color: "var(--err)",
          border: "1px solid oklch(0.68 0.16 25 / 0.3)",
        }
      : variant === "ghost"
      ? {
          background: "transparent",
          color: "var(--fg-1)",
          border: "1px solid transparent",
        }
      : {
          background: "var(--ink-3)",
          color: "var(--fg-0)",
          border: "1px solid var(--hair-2)",
        };

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={
        "rounded-[6px] inline-flex items-center justify-center gap-2 tight font-medium focus-ring transition-colors " +
        sizeCls +
        (fullWidth ? " w-full" : "") +
        (disabled ? " opacity-50 cursor-not-allowed" : "")
      }
      style={styles}
    >
      {icon && (
        <span className={variant === "primary" ? "" : "fg-3"}>{icon}</span>
      )}
      {children}
      {kbd && <span className="kbd ml-1">{kbd}</span>}
    </button>
  );
}
