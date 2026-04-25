import { ReactNode } from "react";
import Kicker from "./Kicker";

interface Props {
  label: string;
  children: ReactNode;
}

export default function Section({ label, children }: Props) {
  return (
    <div>
      <div className="mb-2">
        <Kicker>{label}</Kicker>
      </div>
      {children}
    </div>
  );
}
