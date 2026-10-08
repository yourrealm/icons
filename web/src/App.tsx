import { useEffect, useMemo, useState } from "react";
import { fetchSets, markPath, renditions, SET_TITLES, type SetId, type Sets } from "./api.ts";

const SET_IDS: SetId[] = ["char", "fa", "ph", "shape"];
const CHARS = ["÷", "%", "€", "£", "$", "+", "×", "=", "∞", "★", "♥", "→", "?", "!", "#", "R"];
const GRID_LIMIT = 96;

export function App() {
  const [sets, setSets] = useState<Sets | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [set, setSet] = useState<SetId>("char");
  const [name, setName] = useState("÷");
  const [search, setSearch] = useState("");
  const [grain, setGrain] = useState(0);

  useEffect(() => {
    fetchSets().then(setSets, (e: Error) => setError(e.message));
  }, []);

  const names = useMemo(() => {
    if (!sets || set === "char") return [];
    const q = search.trim().toLowerCase();
    const all = sets[set];
    return (q ? all.filter((n) => n.includes(q)) : all).slice(0, GRID_LIMIT);
  }, [sets, set, search]);

  const pick = (s: SetId) => {
    setSet(s);
    setSearch("");
    setName(s === "char" ? "÷" : s === "shape" ? "divide" : "");
  };

  return (
    <main>
      <header>
        <img src={markPath("char", "R", "svg", { crop: "tile" })} alt="" width={40} height={40} />
        <h1>Realm icons</h1>
        <p>
          Realm's clay mark with any glyph on it. Pick one, download the files, or link to the URL.
        </p>
      </header>

      <section className="picker">
        <nav className="tabs" aria-label="Source">
          {SET_IDS.map((s) => (
            <button
              key={s}
              className={s === set ? "on" : ""}
              onClick={() => pick(s)}
            >
              {SET_TITLES[s]}
            </button>
          ))}
        </nav>

        {set === "char"
          ? (
            <div className="chars">
              <label>
                One character, in Nunito Black
                <input
                  value={name}
                  onChange={(e) => setName(Array.from(e.target.value).slice(-1).join(""))}
                  size={2}
                  autoFocus
                />
              </label>
              <div className="quick">
                {CHARS.map((c) => (
                  <button
                    key={c}
                    className={c === name ? "on" : ""}
                    onClick={() => setName(c)}
                  >
                    {c}
                  </button>
                ))}
              </div>
            </div>
          )
          : (
            <>
              {set !== "shape" && (
                <input
                  className="search"
                  placeholder={`Search ${sets ? sets[set].length : ""} icons`}
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  autoFocus
                />
              )}
              {error && <p className="error">{error}</p>}
              <div className="grid">
                {names.map((n) => (
                  <button
                    key={n}
                    className={n === name ? "on" : ""}
                    title={n}
                    onClick={() => setName(n)}
                  >
                    <img
                      src={markPath(set, n, "svg", { crop: "tile", grain: 0 })}
                      alt={n}
                      loading="lazy"
                    />
                  </button>
                ))}
              </div>
              {sets && names.length === GRID_LIMIT && (
                <p className="hint">First {GRID_LIMIT} matches. Narrow the search.</p>
              )}
            </>
          )}
      </section>

      {name && <Result set={set} name={name} grain={grain} setGrain={setGrain} />}
    </main>
  );
}

type ResultProps = {
  set: SetId;
  name: string;
  grain: number;
  setGrain: (g: number) => void;
};

function Result({ set, name, grain, setGrain }: ResultProps) {
  const q = { grain: grain || undefined };
  const files = renditions(set, name, grain);
  return (
    <section className="result">
      {(set === "ph" || set === "shape") && <OnHome set={set} name={name} />}

      <div className="previews">
        <figure className="light">
          <img src={markPath(set, name, "svg", q)} alt={`${name}, light`} />
          <figcaption>Light</figcaption>
        </figure>
        <figure className="dark">
          <img src={markPath(set, name, "svg", { ...q, theme: "dark" })} alt={`${name}, dark`} />
          <figcaption>Dark</figcaption>
        </figure>
      </div>

      <div className="controls">
        <label>
          Grain <output>{grain.toFixed(1)}</output>
          <input
            type="range"
            min={0}
            max={1}
            step={0.1}
            value={grain}
            onChange={(e) => setGrain(Number(e.target.value))}
          />
        </label>
      </div>

      <table className="files">
        <tbody>
          {files.map((f) => <FileRow key={f.file} file={f.file} path={f.path} />)}
        </tbody>
      </table>
    </section>
  );
}

/// Home draws `ph:` and `shape:` icons itself, from the same mark, so an app
/// names one in realm.tsx instead of linking here. Home's renderer has no
/// grain, so this preview ignores it.
function OnHome({ set, name }: { set: "ph" | "shape"; name: string }) {
  const snippet = `icon: "${set}:${name}",`;
  const [copied, copy] = useCopy(snippet);
  return (
    <div className="home">
      <h2>On Realm Home</h2>
      <div className="home-panels">
        {(["light", "dark"] as const).map((theme) => {
          const src = markPath(set, name, "svg", { crop: "tile", theme });
          return (
            <div key={theme} className={`home-panel ${theme}`}>
              <div className="home-launcher">
                <img src={src} alt="" width={80} height={80} />
                <span>{name}</span>
              </div>
              <div className="home-card">
                <img src={src} alt="" width={52} height={52} />
                <span>
                  <b>{name}</b>
                  <small>All workers</small>
                </span>
              </div>
            </div>
          );
        })}
      </div>
      <table className="files">
        <tbody>
          <tr>
            <th>realm.tsx</th>
            <td>
              <code>{snippet}</code>
            </td>
            <td className="actions">
              <button onClick={copy}>{copied ? "Copied" : "Copy"}</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}

function useCopy(text: string): [boolean, () => Promise<void>] {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return [copied, copy];
}

function FileRow({ file, path }: { file: string; path: string }) {
  const url = `${location.origin}${path}`;
  const [copied, copy] = useCopy(url);
  return (
    <tr>
      <th>{file}</th>
      <td>
        <code>{url}</code>
      </td>
      <td className="actions">
        <button onClick={copy}>{copied ? "Copied" : "Copy URL"}</button>
        <a className="button" href={path} download={file}>Download</a>
      </td>
    </tr>
  );
}
