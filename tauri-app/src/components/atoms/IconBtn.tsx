import { MouseEvent, ReactNode } from "react";

interface Props {
  children: ReactNode;
  onClick?: (e: MouseEvent<HTMLButtonElement>) => void;
  active?: boolean;
  danger?: boolean;
  title?: string;
  className?: string;
}

export default function IconBtn({
  children,
  onClick,
  active = false,
  danger = false,
  title,
  className = "",
}: Props) {
  return (
    <button
      onClick={onClick}
      title={title}
      className={
        "w-7 h-7 rounded-[6px] inline-flex items-center justify-center transition-colors focus-ring " +
        className
      }
      style={{
        background: active ? "var(--ink-3)" : "transparent",
        color: danger
          ? "var(--err)"
          : active
          ? "var(--fg-0)"
          : "var(--fg-2)",
      }}
      onMouseEnter={(e) => {
        if (!active) e.currentTarget.style.background = "var(--ink-3)";
      }}
      onMouseLeave={(e) => {
        if (!active) e.currentTarget.style.background = "transparent";
      }}
    >
      {children}
    </button>
  );
}
