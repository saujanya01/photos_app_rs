import { ReactNode } from "react";

interface SvgProps {
  c: ReactNode;
  s?: number;
  sw?: number;
  cls?: string;
}

const Svg = ({ c, s = 16, sw = 1.5, cls = "" }: SvgProps) => (
  <svg
    width={s}
    height={s}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={sw}
    strokeLinecap="round"
    strokeLinejoin="round"
    className={cls}
  >
    {c}
  </svg>
);

type Icon = (p?: { s?: number; sw?: number; cls?: string }) => JSX.Element;

export const Ic: Record<string, Icon> = {
  search: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <circle cx="11" cy="11" r="7" />
          <path d="m20 20-3.5-3.5" />
        </>
      }
    />
  ),
  image: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <circle cx="9" cy="9" r="2" />
          <path d="m21 15-5-5L5 21" />
        </>
      }
    />
  ),
  video: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <rect x="3" y="6" width="14" height="12" rx="2" />
          <path d="m17 10 4-2v8l-4-2z" />
        </>
      }
    />
  ),
  layers: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="m12 2 9 5-9 5-9-5z" />
          <path d="m3 12 9 5 9-5" />
          <path d="m3 17 9 5 9-5" />
        </>
      }
    />
  ),
  star: (p = {}) => (
    <Svg
      {...p}
      c={
        <path d="m12 3 2.6 5.8 6.4.7-4.8 4.3 1.4 6.3L12 17l-5.6 3.1 1.4-6.3L3 9.5l6.4-.7z" />
      }
    />
  ),
  cam: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M4 7h3l2-3h6l2 3h3a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2z" />
          <circle cx="12" cy="13" r="3.5" />
        </>
      }
    />
  ),
  drive: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <rect x="3" y="4" width="18" height="7" rx="2" />
          <rect x="3" y="13" width="18" height="7" rx="2" />
          <circle cx="7" cy="7.5" r=".6" fill="currentColor" />
          <circle cx="7" cy="16.5" r=".6" fill="currentColor" />
        </>
      }
    />
  ),
  folder: (p = {}) => (
    <Svg
      {...p}
      c={
        <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      }
    />
  ),
  sliders: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M4 6h10M4 12h6M4 18h14" />
          <circle cx="17" cy="6" r="2" />
          <circle cx="13" cy="12" r="2" />
          <circle cx="19" cy="18" r="2" />
        </>
      }
    />
  ),
  x: (p = {}) => <Svg {...p} c={<path d="M6 6l12 12M6 18 18 6" />} />,
  info: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M12 8v.01M11 12h1v5h1" />
        </>
      }
    />
  ),
  aL: (p = {}) => <Svg {...p} c={<path d="M15 18 9 12l6-6" />} />,
  aR: (p = {}) => <Svg {...p} c={<path d="m9 6 6 6-6 6" />} />,
  plus: (p = {}) => <Svg {...p} c={<path d="M12 5v14M5 12h14" />} />,
  grid: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <rect x="3" y="3" width="7" height="7" rx="1" />
          <rect x="14" y="3" width="7" height="7" rx="1" />
          <rect x="3" y="14" width="7" height="7" rx="1" />
          <rect x="14" y="14" width="7" height="7" rx="1" />
        </>
      }
    />
  ),
  map: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M9 4 3 7v13l6-3 6 3 6-3V4l-6 3z" />
          <path d="M9 4v13M15 7v13" />
        </>
      }
    />
  ),
  tag: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M12 3H5a2 2 0 0 0-2 2v7a2 2 0 0 0 .6 1.4l8 8a2 2 0 0 0 2.8 0l7-7a2 2 0 0 0 0-2.8l-8-8A2 2 0 0 0 12 3z" />
          <circle
            cx="8"
            cy="8"
            r=".01"
            fill="currentColor"
            stroke="currentColor"
            strokeWidth={2}
          />
        </>
      }
    />
  ),
  check: (p = {}) => <Svg {...p} c={<path d="m5 12 4 4L19 6" />} />,
  chevD: (p = {}) => <Svg {...p} c={<path d="m6 9 6 6 6-6" />} />,
  play: (p = {}) => (
    <Svg {...p} c={<path d="M6 4v16l14-8z" fill="currentColor" stroke="none" />} />
  ),
  upload: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <path d="M17 8l-5-5-5 5" />
          <path d="M12 3v12" />
        </>
      }
    />
  ),
  scan: (p = {}) => (
    <Svg
      {...p}
      c={
        <path d="M4 8V5a1 1 0 0 1 1-1h3M20 8V5a1 1 0 0 0-1-1h-3M4 16v3a1 1 0 0 0 1 1h3M20 16v3a1 1 0 0 1-1 1h-3M7 12h10" />
      }
    />
  ),
  pin: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <path d="M12 22s-8-8-8-13a8 8 0 1 1 16 0c0 5-8 13-8 13z" />
          <circle cx="12" cy="9" r="3" />
        </>
      }
    />
  ),
  gear: (p = {}) => (
    <Svg
      {...p}
      c={
        <>
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 0 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.6 1.7 1.7 0 0 0-1.8.4l-.1.1a2 2 0 0 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.1a1.7 1.7 0 0 0 1.6-1.1 1.7 1.7 0 0 0-.4-1.8l-.1-.1a2 2 0 0 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 0 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
        </>
      }
    />
  ),
};
