import { MouseEvent, ReactNode } from "react";

interface Props {
  children: ReactNode;
  icon?: ReactNode;
  onClick?: (e: MouseEvent<HTMLButtonElement>) => void;
  active?: boolean;
  title?: string;
}

export default function Chip({
  children,
  icon,
  onClick,
  active = false,
  title,
}: Props) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="h-7 px-2 rounded-[6px] inline-flex items-center gap-1.5 text-[12px] transition-colors focus-ring"
      style={{
        background: active ? "var(--accent-soft)" : "transparent",
        color: active ? "var(--accent)" : "var(--fg-2)",
        border: active ? "1px solid var(--accent-dim)" : "1px solid transparent",
      }}
      onMouseEnter={(e) => {
        if (!active) e.currentTarget.style.background = "var(--ink-3)";
      }}
      onMouseLeave={(e) => {
        if (!active) e.currentTarget.style.background = "transparent";
      }}
    >
      {icon && <span className="fg-4">{icon}</span>}
      {children}
    </button>
  );
}
