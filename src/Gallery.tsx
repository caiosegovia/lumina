import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useWindowVirtualizer } from "@tanstack/react-virtual";
import {
  AlertTriangle,
  CalendarDays,
  Check,
  ChevronDown,
  Copy,
  Grid3X3,
  HardDrive,
  Images,
  List,
  LoaderCircle,
  Rows3,
  Search,
  Tags,
  Video,
  X,
} from "lucide-react";
import { api } from "./api";
import { formatBytes } from "./format";
import type { Album, GalleryFilters, GalleryResult, MediaAsset } from "./types";
const thumbs = new Map<string, string | null>(),
  empty: GalleryFilters = { query: "" };
type Mode = "grid" | "list";
type Group = "day" | "month" | "year";
type Zoom = "compact" | "normal" | "large";
const session: {
  filters: GalleryFilters;
  result?: GalleryResult;
  assets: MediaAsset[];
  scrollY: number;
} = { filters: empty, assets: [], scrollY: 0 };
const saved = <T extends string>(key: string, fallback: T) =>
  (localStorage.getItem(key) || fallback) as T;
export function resetGallerySession() {
  session.filters = empty;
  session.result = undefined;
  session.assets = [];
  session.scrollY = 0;
  thumbs.clear();
}
export default function Gallery() {
  const [filters, setFilters] = useState(session.filters),
    [draft, setDraft] = useState(session.filters),
    [result, setResult] = useState<GalleryResult | undefined>(session.result),
    [assets, setAssets] = useState(session.assets),
    [filterOpen, setFilterOpen] = useState(false),
    [preview, setPreview] = useState<MediaAsset>(),
    [loading, setLoading] = useState(false),
    [error, setError] = useState("");
  const [mode, setMode0] = useState<Mode>(() => saved("lumina-view", "grid")),
    [group, setGroup0] = useState<Group>(() => saved("lumina-group", "month")),
    [zoom, setZoom0] = useState<Zoom>(() => saved("lumina-zoom", "normal")),
    [selection, setSelection] = useState<Set<string>>(new Set()),
    [action, setAction] = useState<"tag" | "album" | "date">(),
    [notice, setNotice] = useState(""),
    [refresh, setRefresh] = useState(0);
  const seq = useRef(0),
    width = { compact: 145, normal: 190, large: 260 }[zoom],
    [columns, setColumns] = useState(4),
    signature = JSON.stringify(filters) + refresh;
  const saveMode = (x: Mode) => {
      localStorage.setItem("lumina-view", x);
      setMode0(x);
    },
    saveGroup = (x: Group) => {
      localStorage.setItem("lumina-group", x);
      setGroup0(x);
    },
    saveZoom = (x: Zoom) => {
      localStorage.setItem("lumina-zoom", x);
      setZoom0(x);
    };
  const load = useCallback(
    async (cursor?: string) => {
      const id = ++seq.current;
      setLoading(true);
      setError("");
      try {
        const page = await api.gallery(filters, cursor, 100);
        if (id !== seq.current) return;
        setResult((previous) =>
          cursor && previous
            ? { ...page, summary: previous.summary, options: previous.options }
            : page,
        );
        setAssets((old) => (cursor ? [...old, ...page.assets] : page.assets));
      } catch (e) {
        if (id === seq.current) setError(String(e));
      } finally {
        if (id === seq.current) setLoading(false);
      }
    },
    [signature],
  );
  useEffect(() => {
    const t = setTimeout(() => load(), 200);
    return () => clearTimeout(t);
  }, [load]);
  useEffect(() => {
    session.filters = filters;
    session.result = result;
    session.assets = assets;
  }, [filters, result, assets]);
  useEffect(() => {
    requestAnimationFrame(() => scrollTo({ top: session.scrollY }));
    return () => {
      session.scrollY = scrollY;
    };
  }, []);
  useEffect(() => {
    const resize = () =>
      setColumns(Math.max(1, Math.floor((innerWidth - 330) / width)));
    resize();
    addEventListener("resize", resize);
    return () => removeEventListener("resize", resize);
  }, [width]);
  const groups = useMemo(() => {
    const m = new Map<string, { label: string; items: MediaAsset[] }>();
    assets.forEach((a) => {
      const d = new Date(a.capturedAt),
        key =
          group === "year"
            ? a.capturedAt.slice(0, 4)
            : group === "month"
              ? a.capturedAt.slice(0, 7)
              : a.capturedAt.slice(0, 10),
        formattedLabel = new Intl.DateTimeFormat(
          "pt-BR",
          group === "year"
            ? { year: "numeric" }
            : group === "month"
              ? { month: "long", year: "numeric" }
              : { dateStyle: "long" },
        ).format(d),
        label = formattedLabel.charAt(0).toLocaleUpperCase("pt-BR") + formattedLabel.slice(1);
      if (!m.has(key)) m.set(key, { label, items: [] });
      m.get(key)!.items.push(a);
    });
    return [...m.entries()];
  }, [assets, group]);
  const rows = useMemo(
    () =>
      groups.flatMap(([key, g]) => [
        {
          kind: "header" as const,
          key: `h${key}`,
          label: g.label,
          count: g.items.length,
        },
        ...(mode === "list"
          ? g.items.map((a) => ({
              kind: "items" as const,
              key: a.id,
              items: [a],
            }))
          : Array.from(
              { length: Math.ceil(g.items.length / columns) },
              (_, i) => ({
                kind: "items" as const,
                key: `${key}-${i}`,
                items: g.items.slice(i * columns, (i + 1) * columns),
              }),
            )),
      ]),
    [groups, mode, columns],
  );
  const virtual = useWindowVirtualizer({
      count: rows.length,
      estimateSize: (i) =>
        rows[i]?.kind === "header"
          ? 48
          : mode === "list"
            ? 70
            : { compact: 175, normal: 225, large: 295 }[zoom],
      overscan: 5,
      getItemKey: (i) => rows[i]?.key || i,
    }),
    visible = virtual.getVirtualItems();
  useEffect(() => {
    const last = visible.at(-1);
    if (last && last.index >= rows.length - 3 && result?.nextCursor && !loading)
      load(result.nextCursor);
  }, [
    visible.map((v) => v.index).join(),
    rows.length,
    result?.nextCursor,
    loading,
    load,
  ]);
  const toggle = (id: string) =>
      setSelection((old) => {
        const n = new Set(old);
        n.has(id) ? n.delete(id) : n.add(id);
        return n;
      }),
    active = Object.entries(filters).filter(
      ([k, v]) => k !== "query" && v !== undefined && v !== "",
    ).length,
    s = result?.summary,
    clear = () => {
      setDraft(empty);
      setFilters(empty);
    };
  return (
    <>
      <div className="gallery-stats">
        <Metric
          label="Mídias"
          value={(s?.total || 0).toLocaleString("pt-BR")}
        />
        <Metric label="Espaço" value={formatBytes(s?.bytes || 0)} />
        <Metric
          label="Protegidas"
          value={
            s?.total ? `${Math.round((s.protected / s.total) * 100)}%` : "0%"
          }
        />
        <Metric
          label="Com localização"
          value={(s?.withLocation || 0).toLocaleString("pt-BR")}
        />
        <Metric
          label="Em várias origens"
          value={(s?.duplicateAssets || 0).toLocaleString("pt-BR")}
        />
      </div>
      <div className="year-strip">
        {s?.years.map((y) => (
          <button
            key={y.year}
            className={filters.year === +y.year ? "active" : ""}
            onClick={() =>
              setFilters((v) => ({
                ...v,
                year: v.year === +y.year ? undefined : +y.year,
              }))
            }
          >
            <strong>{y.year}</strong>
            <span>
              {y.count.toLocaleString("pt-BR")} · {formatBytes(y.bytes)}
            </span>
          </button>
        ))}
      </div>
      {active > 0 && (
        <div className="filter-chips">
          <span>{result?.matched || 0} resultados</span>
          <button onClick={clear}>Limpar filtros</button>
        </div>
      )}
      <div className="toolbar gallery-toolbar">
        <div className="search">
          <Search />
          <input
            aria-label="Buscar na galeria"
            placeholder="Buscar por nome, câmera ou tag…"
            value={filters.query}
            onChange={(e) =>
              setFilters((v) => ({ ...v, query: e.target.value }))
            }
          />
        </div>
        <ChoiceMenu icon={<CalendarDays/>} label="Agrupar" value={group} options={[{value:"day",label:"Por dia"},{value:"month",label:"Por mês"},{value:"year",label:"Por ano"}]} onChange={v=>saveGroup(v as Group)}/>
        {mode === "grid" && (
          <ChoiceMenu icon={<Rows3/>} label="Tamanho da grade" value={zoom} options={[{value:"compact",label:"Compacta"},{value:"normal",label:"Confortável"},{value:"large",label:"Ampla"}]} onChange={v=>saveZoom(v as Zoom)}/>
        )}
        <div className="view-switch">
          <button
            aria-label="Visão em grade"
            className={mode === "grid" ? "active" : ""}
            onClick={() => saveMode("grid")}
          >
            <Grid3X3 />
          </button>
          <button
            aria-label="Visão em lista"
            className={mode === "list" ? "active" : ""}
            onClick={() => saveMode("list")}
          >
            <List />
          </button>
        </div>
        <button
          className={`filter ${active ? "active" : ""}`}
          onClick={() => {
            setDraft(filters);
            setFilterOpen((v) => !v);
          }}
        >
          <Tags /> Filtros {active > 0 && <b>{active}</b>}
        </button>
      </div>
      {filterOpen && (
        <Filters
          value={draft}
          options={result?.options}
          change={setDraft}
          clear={clear}
          apply={() => {
            setFilters(draft);
            setFilterOpen(false);
          }}
        />
      )}
      {notice && (
        <p className="safe-note" role="status">
          {notice}
        </p>
      )}
      {selection.size > 0 && (
        <div className="bulk-bar">
          <strong>{selection.size} selecionadas</strong>
          <button onClick={() => setAction("tag")}>Aplicar tag</button>
          <button onClick={() => setAction("album")}>Adicionar ao álbum</button>
          <button onClick={() => setAction("date")}>Corrigir data</button>
          <button onClick={() => setSelection(new Set())}>Limpar</button>
        </div>
      )}
      {error && (
        <div className="notice error">
          {error}
          <button onClick={() => load()}>Tentar novamente</button>
        </div>
      )}
      {!loading && !assets.length ? (
        <div className="empty-gallery">
          <Images />
          <h2>Nenhuma mídia encontrada</h2>
          <p>Remova filtros ou importe uma fonte.</p>
        </div>
      ) : (
        <div
          className={`virtual-gallery ${mode}`}
          style={{ height: virtual.getTotalSize(), position: "relative" }}
        >
          {visible.map((v) => {
            const row = rows[v.index];
            return (
              <div
                key={row.key}
                ref={virtual.measureElement}
                data-index={v.index}
                className={`virtual-row ${row.kind}`}
                style={{
                  position: "absolute",
                  transform: `translateY(${v.start}px)`,
                  width: "100%",
                  ...(row.kind === "items" && mode === "grid"
                    ? {
                        display: "grid",
                        gridTemplateColumns: `repeat(${columns},minmax(0,1fr))`,
                      }
                    : {}),
                }}
              >
                {row.kind === "header" ? (
                  <h3>
                    {row.label} <span>{row.count}</span>
                  </h3>
                ) : (
                  row.items.map((a) => (
                    <Item
                      key={a.id}
                      asset={a}
                      mode={mode}
                      checked={selection.has(a.id)}
                      toggle={() => toggle(a.id)}
                      open={() =>
                        selection.size ? toggle(a.id) : setPreview(a)
                      }
                    />
                  ))
                )}
              </div>
            );
          })}
        </div>
      )}
      {loading && (
        <div className="gallery-loading">
          <LoaderCircle className="spin" /> Carregando mídias…
        </div>
      )}
      {!result?.nextCursor && !!assets.length && (
        <p className="gallery-end">
          Todas as {result?.matched.toLocaleString("pt-BR")} mídias carregadas
        </p>
      )}
      {preview && (
        <Preview asset={preview} close={() => setPreview(undefined)} />
      )}{" "}
      {action && (
        <Bulk
          action={action}
          assets={assets.filter((a) => selection.has(a.id))}
          close={() => setAction(undefined)}
          done={(x) => {
            setNotice(x);
            setSelection(new Set());
            setAction(undefined);
            setRefresh((v) => v + 1);
          }}
        />
      )}
    </>
  );
}
function ChoiceMenu({icon,label,value,options,onChange}:{icon:React.ReactNode;label:string;value:string;options:{value:string;label:string}[];onChange:(value:string)=>void}){const[open,setOpen]=useState(false),selected=options.find(x=>x.value===value)?.label;return <div className="choice-menu"><button aria-label={label} aria-haspopup="listbox" aria-expanded={open} onClick={()=>setOpen(v=>!v)}>{icon}<span>{selected}</span><ChevronDown/></button>{open&&<div className="choice-popover" role="listbox" aria-label={label}>{options.map(option=><button role="option" aria-selected={option.value===value} className={option.value===value?"active":""} key={option.value} onClick={()=>{onChange(option.value);setOpen(false)}}>{option.label}{option.value===value&&<Check/>}</button>)}</div>}</div>}
function Item({
  asset,
  mode,
  checked,
  toggle,
  open,
}: {
  asset: MediaAsset;
  mode: Mode;
  checked: boolean;
  toggle: () => void;
  open: () => void;
}) {
  return (
    <div className={`media-card selectable ${checked ? "selected" : ""}`}>
      <button
        className="selection-check"
        aria-label={`Selecionar ${asset.filename}`}
        aria-pressed={checked}
        onClick={toggle}
      >
        {checked && <Check />}
      </button>
      <button
        className="media-main"
        aria-label={`Abrir detalhes de ${asset.filename}`}
        onClick={open}
      >
        <MediaThumb asset={asset} />
        <div>
          <strong>{asset.filename}</strong>
          <small>{asset.camera || "Dispositivo desconhecido"}</small>
        </div>
        {asset.dateSuspicious && (
          <span className="suspicious">
            <AlertTriangle /> Data a revisar
          </span>
        )}
        {mode === "grid" && asset.protectionState === "error" && <span className="asset-state error">Revisar proteção</span>}
        {mode === "list" && (
          <>
            <time>{new Date(asset.capturedAt).toLocaleString("pt-BR")}</time>
            <span>
              {asset.extension.toUpperCase()} · {formatBytes(asset.bytes)}
            </span>
            <span>{asset.sourceNames.join(", ") || "Acervo"}</span>
            <span className="protection">
              {asset.protectionState.replaceAll("_", " ")}
            </span>
          </>
        )}
      </button>
    </div>
  );
}
function Bulk({
  action,
  assets,
  close,
  done,
}: {
  action: "tag" | "album" | "date";
  assets: MediaAsset[];
  close: () => void;
  done: (x: string) => void;
}) {
  const [value, setValue] = useState(""),
    [albums, setAlbums] = useState<Album[]>([]),
    [error, setError] = useState("");
  useEffect(() => {
    if (action === "album") api.albums().then(setAlbums);
  }, [action]);
  const submit = async () => {
    try {
      const ids = assets.map((a) => a.id),
        r =
          action === "tag"
            ? await api.applyTag(value, ids)
            : action === "album"
              ? await api.addToAlbum(value, ids)
              : await api.updateCaptureDate(ids, new Date(value).toISOString());
      done(`${r.affected} mídias atualizadas`);
    } catch (e) {
      setError(String(e));
    }
  };
  return (
    <div className="modal-backdrop">
      <div className="modal compact">
        <button className="icon-only close" onClick={close}>
          <X />
        </button>
        <h2>
          {action === "tag"
            ? "Aplicar tag"
            : action === "album"
              ? "Adicionar ao álbum"
              : "Corrigir data de captura"}
        </h2>
        <p>
          Aplicar a {assets.length} mídias apenas no catálogo; os originais não
          serão alterados.
        </p>
        {action === "album" ? (
          <select
            aria-label="Álbum"
            value={value}
            onChange={(e) => setValue(e.target.value)}
          >
            <option value="">Escolha um álbum</option>
            {albums.map((a) => (
              <option key={a.id} value={a.id}>
                {a.name}
              </option>
            ))}
          </select>
        ) : (
          <input
            aria-label={action === "tag" ? "Nome da tag" : "Nova data"}
            type={action === "date" ? "datetime-local" : "text"}
            value={value}
            onChange={(e) => setValue(e.target.value)}
          />
        )}{" "}
        {error && <p className="error">{error}</p>}
        <div className="modal-actions">
          <button onClick={close}>Cancelar</button>
          <button className="primary" disabled={!value} onClick={submit}>
            Aplicar
          </button>
        </div>
      </div>
    </div>
  );
}
function Metric({ label, value }: { label: string; value: string }) {
  return (
    <article>
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}
function Select({
  label,
  value,
  values = [],
  change,
}: {
  label: string;
  value?: string;
  values?: { value: string; label: string; count: number }[];
  change: (x: string) => void;
}) {
  return (
    <label>
      {label}
      <select value={value || ""} onChange={(e) => change(e.target.value)}>
        <option value="">Todos</option>
        {values.map((x) => (
          <option key={x.value} value={x.value}>
            {x.label}
            {x.count ? ` (${x.count})` : ""}
          </option>
        ))}
      </select>
    </label>
  );
}
function Filters({
  value,
  options,
  change,
  clear,
  apply,
}: {
  value: GalleryFilters;
  options?: GalleryResult["options"];
  change: (x: GalleryFilters) => void;
  clear: () => void;
  apply: () => void;
}) {
  const set = (k: keyof GalleryFilters, v: string) =>
    change({ ...value, [k]: v || undefined });
  return (
    <div className="filter-panel">
      <label>
        De
        <input
          type="date"
          value={value.dateFrom || ""}
          onChange={(e) => set("dateFrom", e.target.value)}
        />
      </label>
      <label>
        Até
        <input
          type="date"
          value={value.dateTo || ""}
          onChange={(e) => set("dateTo", e.target.value)}
        />
      </label>
      <Select
        label="Tipo"
        value={value.mediaType}
        values={[
          { value: "photo", label: "Fotos", count: 0 },
          { value: "video", label: "Vídeos", count: 0 },
          { value: "raw", label: "RAW", count: 0 },
        ]}
        change={(v) => set("mediaType", v)}
      />
      <Select
        label="Câmera"
        value={value.camera}
        values={options?.cameras}
        change={(v) => set("camera", v)}
      />
      <Select
        label="Fonte"
        value={value.sourceId}
        values={options?.sources}
        change={(v) => set("sourceId", v)}
      />
      <Select
        label="Extensão"
        value={value.extension}
        values={options?.extensions}
        change={(v) => set("extension", v)}
      />
      <Select
        label="Tag"
        value={value.tagId}
        values={options?.tags}
        change={(v) => set("tagId", v)}
      />
      <Select
        label="Álbum"
        value={value.albumId}
        values={options?.albums}
        change={(v) => set("albumId", v)}
      />
      <label>
        Qualidade da data
        <select
          value={value.dateSuspicious ? "true" : ""}
          onChange={(e) =>
            change({
              ...value,
              dateSuspicious: e.target.value ? true : undefined,
            })
          }
        >
          <option value="">Todas</option>
          <option value="true">Datas a revisar</option>
        </select>
      </label>
      <div className="filter-actions">
        <button onClick={clear}>Limpar</button>
        <button className="primary" onClick={apply}>
          Aplicar filtros
        </button>
      </div>
    </div>
  );
}
export function MediaThumb({
  asset,
  className = "",
}: {
  asset: MediaAsset;
  className?: string;
}) {
  const [src, setSrc] = useState<string | null | undefined>(() =>
    thumbs.get(asset.id),
  );
  useEffect(() => {
    if (src === null) {
      const retry = window.setTimeout(() => setSrc(undefined), 750);
      return () => window.clearTimeout(retry);
    }
    if (src !== undefined) return;
    let live = true;
    api
      .thumbnail(asset.id)
      .then((x) => {
        if (live) {
          thumbs.set(asset.id, x);
          setSrc(x);
        }
      })
      .catch(() => live && setSrc(null));
    return () => {
      live = false;
    };
  }, [asset.id, src]);
  return (
    <div className={`media-placeholder ${asset.mediaType} ${className}`}>
      {src ? (
        <img
          src={src}
          alt={`Prévia de ${asset.filename}`}
          loading="lazy"
          onError={() => {
            thumbs.delete(asset.id);
            setSrc(undefined);
          }}
        />
      ) : (
        <span>{asset.mediaType === "video" ? <Video /> : <Images />}</span>
      )}
      {asset.mediaType === "video" && <i>VÍDEO</i>}
      {asset.occurrenceCount > 1 && (
        <b>
          <Copy /> {asset.occurrenceCount}
        </b>
      )}
    </div>
  );
}
function Preview({ asset, close }: { asset: MediaAsset; close: () => void }) {
  return (
    <div className="drawer">
      <button
        aria-label="Fechar detalhes"
        className="icon-only close"
        onClick={close}
      >
        <X />
      </button>
      <MediaThumb asset={asset} className="drawer-preview" />
      <h2>{asset.filename}</h2>
      <p>{new Date(asset.capturedAt).toLocaleString("pt-BR")}</p>
      {asset.dateSuspicious && (
        <p className="notice warning">
          <AlertTriangle /> Data a revisar
        </p>
      )}
      <hr />
      <Info
        label="Tipo"
        value={`${asset.mediaType} · ${asset.extension.toUpperCase()}`}
      />
      <Info label="Tamanho" value={formatBytes(asset.bytes)} />
      <Info label="Câmera" value={asset.camera || "—"} />
      <Info label="Data obtida de" value={asset.dateSource} />
      <hr />
      <p className="eyebrow">LOCALIZAÇÕES</p>
      <div className="location good">
        <HardDrive />
        <div>
          <strong>Acervo mestre</strong>
          <small>{asset.masterPath}</small>
        </div>
      </div>
      {asset.sourceNames.map((s) => (
        <div className="location" key={s}>
          <HardDrive />
          <div>
            <strong>{s}</strong>
            <small>Fonte original</small>
          </div>
        </div>
      ))}
      <hr />
      <p className="hash">
        SHA-256
        <br />
        <code>{asset.hash}</code>
      </p>
    </div>
  );
}
function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
