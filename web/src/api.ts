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
export function renditions(set: SetId, name: string, grain: number, label: string) {
  const g = grain === 1 ? undefined : grain;
  const l = label || undefined;
  return [
    { file: "logo.svg", path: markPath(set, name, "svg", { grain: g, label: l }) },
    {
      file: "logo-dark.svg",
      path: markPath(set, name, "svg", { theme: "dark", grain: g, label: l }),
    },
    {
      file: "favicon.svg",
      path: markPath(set, name, "svg", { theme: "auto", crop: "tile", grain: g, label: l }),
    },
    { file: "favicon.ico", path: markPath(set, name, "ico", { label: l }) },
    { file: "icon-512.png", path: markPath(set, name, "png", { size: 512, grain: g, label: l }) },
  ];
}
