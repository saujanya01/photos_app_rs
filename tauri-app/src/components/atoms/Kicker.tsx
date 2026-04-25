import { ReactNode } from "react";

export default function Kicker({ children }: { children: ReactNode }) {
  return <div className="kicker">{children}</div>;
}
