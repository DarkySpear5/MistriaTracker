import type { CSSProperties } from "react";
const paths: Record<string, string> = {
  overview: "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
  museum: "M3 9h18L12 3 3 9z M5 11v8m5-8v8m4-8v8m5-8v8M3 21h18",
  villagers:
    "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M19 8a3 3 0 0 1 0 6m3 7v-2a4 4 0 0 0-3-4 M12 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0",
  encyclopedia:
    "M12 5C8 2 4 3 2 4v16c3-1 7-1 10 1 3-2 7-2 10-1V4c-2-1-6-2-10 1z M12 5v16",
  settings:
    "M9 3h6l1 4 4 1 2 5-3 3 1 4-5 2-3-3-4 1-2-5 3-3-1-4z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
  fish: "M3 12c5-9 12-9 17 0-5 9-12 9-17 0z M3 12 1 7v10z M16 10h.01 M10 5l-1-3m1 17-1 3",
  bugs: "M8 8h8v9a4 4 0 0 1-8 0z M10 8V5h4v3 M5 10h3m8 0h3M4 15h4m8 0h4M6 21l3-3m6 0 3 3M9 2l2 3m4-3-2 3M12 9v12",
  crops:
    "M12 21V9 M12 13C3 14 2 8 3 5c6 0 9 3 9 8z M12 9c0-6 4-7 9-7 0 5-3 8-9 7z",
  artifacts:
    "M8 3h8v4l4 6v5a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3v-5l4-6z M8 7h8M4 14h16M9 17h6",
  dishes: "M2 16h20M4 15a8 8 0 0 1 16 0M12 7V4m-2 0h4M4 20h16",
  recipes:
    "M7 4a3 3 0 0 0-5 2c0 2 2 3 4 3v11h14V5a3 3 0 0 0-3-3H5 M10 8h6m-6 4h6m-6 4h4",
  furniture:
    "M4 12V7a3 3 0 0 1 3-3h10a3 3 0 0 1 3 3v5M2 12h20v7H2z M5 19v3m14-3v3M6 12V9m12 3V9",
  blacksmithing: "M3 6h18l-4 5h-4v6h5v4H6v-4h4v-6H6z",
  materials: "M12 2 3 8v9l9 5 9-5V8z M3 8l9 5 9-5 M12 13v9",
  ranching: "M3 11 12 3l9 8v10H3z M9 21v-9h6v9M7 7h10",
  songs:
    "M9 18V5l12-3v13 M9 8l12-3 M9 18a3 3 0 1 1-3-3h3M21 15a3 3 0 1 1-3-3h3",
  perks: "m12 2 3 6 7 1-5 5 1 8-6-4-6 4 1-8-5-5 7-1z",
  scrolls: "M6 2h12v17l-3 3H5a3 3 0 0 1 0-6h13M9 6h5m-5 4h5",
  heart: "M20 4c-3-3-6 0-8 2-2-2-5-5-8-2-4 4-1 9 8 16 9-7 12-12 8-16z",
  search: "M20 20l-5-5 M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0",
  filter: "M3 6h18M6 12h12M9 18h6 M7 4v4m10 2v4m-5 2v4",
  chevron: "m9 5 7 7-7 7",
  back: "m14 5-7 7 7 7",
  close: "m6 6 12 12M18 6 6 18",
  check: "m5 12 4 4L19 6",
  eye: "M2 12c5-9 15-9 20 0-5 9-15 9-20 0zM15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
  lock: "M6 10h12v11H6z M8 10V6a4 4 0 0 1 8 0v4 M12 14v3",
  moon: "M20 15A9 9 0 0 1 9 3a9 9 0 1 0 11 12",
  spark: "m12 2 2 7 7 3-7 2-2 8-2-8-8-2 8-3z",
  leaf: "M5 19C-1 5 12 3 21 3c0 11-2 19-13 16M4 22 17 8",
  note: "M4 3h16v13l-5 5H4zM15 21v-5h5M8 7h8m-8 4h8",
  person: "M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0M3 22c0-12 18-12 18 0z",
  info: "M12 11v6m0-10h.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0",
};
const aliases: Record<string, string> = {
  flora: "leaf",
  forageables: "leaf",
  archaeology: "artifacts",
  insect: "bugs",
  dating: "heart",
  invitations: "heart",
  animals: "ranching",
  cosmetics: "spark",
  other: "materials",
};
export function Icon({
  name,
  size = 20,
  className = "",
  style,
}: {
  name: string;
  size?: number;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <svg
      className={`ui-icon ${className}`}
      style={style}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[aliases[name] ?? name] ?? paths.spark} />
    </svg>
  );
}
