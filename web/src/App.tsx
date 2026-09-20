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
  const [grain, setGrain] = useState(1);
  const [label, setLabel] = useState("");

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

      {name && (
        <Result
          set={set}
          name={name}
          grain={grain}
          setGrain={setGrain}
          label={label}
          setLabel={setLabel}
        />
      )}
    </main>
  );
}

type ResultProps = {
  set: SetId;
  name: string;
  grain: number;
  setGrain: (g: number) => void;
  label: string;
  setLabel: (l: string) => void;
};

function Result({ set, name, grain, setGrain, label, setLabel }: ResultProps) {
  const q = { grain: grain === 1 ? undefined : grain, label: label || undefined };
  const files = renditions(set, name, grain, label);
  return (
    <section className="result">
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
        <label>
          Label
          <input
            value={label}
            placeholder={name}
            maxLength={64}
            onChange={(e) => setLabel(e.target.value)}
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

function FileRow({ file, path }: { file: string; path: string }) {
  const [copied, setCopied] = useState(false);
  const url = `${location.origin}${path}`;
  const copy = async () => {
    await navigator.clipboard.writeText(url);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
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
