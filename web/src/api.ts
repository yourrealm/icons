export type SetId = "char" | "fa" | "ph" | "shape";

export type Sets = {
  char: { font: string };
  fa: string[];
  ph: string[];
  shape: string[];
};

export const SET_TITLES: Record<SetId, string> = {
  char: "Character",
  fa: "Font Awesome",
  ph: "Phosphor",
  shape: "Shapes",
};

export async function fetchSets(): Promise<Sets> {
  const res = await fetch("/api/sets");
  if (!res.ok) throw new Error(`sets: ${res.status}`);
  return res.json();
}

type Query = Record<string, string | number | undefined>;

/// The path of one rendition; the server's URL scheme in one place.
export function markPath(
  set: SetId,
  name: string,
  ext: "svg" | "png" | "ico",
  q: Query = {},
): string {
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(q)) {
    if (v !== undefined && v !== "") params.set(k, String(v));
  }
  const s = params.toString();
  return `/${set}/${encodeURIComponent(name)}.${ext}${s ? `?${s}` : ""}`;
}

/// The files an app ships, in the order the other Realm apps keep them.
export function renditions(set: SetId, name: string, grain: number) {
  const g = grain || undefined;
  return [
    { file: "logo.svg", path: markPath(set, name, "svg", { grain: g }) },
    { file: "logo-dark.svg", path: markPath(set, name, "svg", { theme: "dark", grain: g }) },
    {
      file: "favicon.svg",
      path: markPath(set, name, "svg", { theme: "auto", crop: "tile", grain: g }),
    },
    { file: "favicon.ico", path: markPath(set, name, "ico") },
    { file: "icon-512.png", path: markPath(set, name, "png", { size: 512, grain: g }) },
  ];
}
