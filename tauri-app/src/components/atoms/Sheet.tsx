import { ReactNode, useEffect } from "react";
import IconBtn from "./IconBtn";
import { Ic } from "./icons";

interface Props {
  open?: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  width?: number;
}

/**
 * v0.4 rule: backdrop click does NOT close. Only Esc or the explicit
 * close button. Touchpad misfires were the reason — this is load-bearing.
 */
export default function Sheet({
  open = true,
  onClose,
  title,
  children,
  width = 560,
}: Props) {
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center"
      style={{
        background: "oklch(0 0 0 / 0.5)",
        backdropFilter: "blur(3px)",
      }}
    >
      <div
        className="rounded-[10px] overflow-hidden flex flex-col slide-up"
        style={{
          background: "var(--ink-1)",
          border: "1px solid var(--hair-2)",
          width,
          maxHeight: "86%",
          boxShadow: "0 40px 80px -20px oklch(0 0 0 / 0.7)",
        }}
      >
        <div className="h-11 px-4 flex items-center border-b hair">
          <div className="text-[13px] tight fg-0 font-medium">{title}</div>
          <div className="ml-auto flex items-center gap-2">
            <span className="kbd">ESC</span>
            <IconBtn onClick={onClose}>{Ic.x({ s: 14 })}</IconBtn>
          </div>
        </div>
        <div className="flex-1 overflow-auto scroll p-5">{children}</div>
      </div>
    </div>
  );
}
