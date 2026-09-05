import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useWindowVirtualizer } from "@tanstack/react-virtual";
import {
  AlertTriangle,
  CalendarDays,
  Check,
  ChevronLeft,
  ChevronDown,
  ChevronRight,
  Copy,
  Grid3X3,
  HardDrive,
  Images,
  List,
  LoaderCircle,
  Maximize2,
  Minimize2,
  Rows3,
  Search,
  Star,
  Bookmark,
  Save,
  Tags,
  Video,
  ZoomIn,
  ZoomOut,
  X,
} from "lucide-react";
import { api } from "./api";
import { formatBytes } from "./format";
import type { Album, AssetDetails, GalleryFilters, GalleryResult, GallerySort, MediaAsset, SavedView } from "./types";
const thumbs = new Map<string, string | null>(),
  empty: GalleryFilters = { query: "" };
type Mode = "grid" | "list";
type Group = "day" | "month" | "year";
type Zoom = "compact" | "normal" | "large";
type ListDensity = "compact" | "comfortable";
const session: {
  filters: GalleryFilters;
  result?: GalleryResult;
  assets: MediaAsset[];
  scrollY: number;
  selected: string[];
  compare: boolean;
} = { filters: empty, assets: [], scrollY: 0, selected: [], compare: false };
const saved = <T extends string>(key: string, fallback: T) =>
  (localStorage.getItem(key) || fallback) as T;
export function resetGallerySession() {
  session.filters = empty;
  session.result = undefined;
  session.assets = [];
  session.scrollY = 0;
  session.selected = [];
  session.compare = false;
  thumbs.clear();
}
export function openGalleryWithFilters(filters: Partial<GalleryFilters>) {
  session.filters = { ...empty, ...filters };
  session.result = undefined;
  session.assets = [];
  session.scrollY = 0;
  session.selected = [];
  session.compare = false;
}
export function openGalleryComparison(assetIds:string[]) {
  session.filters = empty;
  session.result = undefined;
  session.assets = [];
  session.scrollY = 0;
  session.selected = assetIds.slice(0,2);
  session.compare = session.selected.length === 2;
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
    [sort, setSort0] = useState<GallerySort>(() => saved("lumina-sort", "captured_desc")),
    [listDensity, setListDensity0] = useState<ListDensity>(() => saved("lumina-list-density", "comfortable")),
    [selection, setSelection] = useState<Set<string>>(()=>new Set(session.selected)),
    [action, setAction] = useState<"tag" | "album" | "date">(),
    [comparing, setComparing] = useState(()=>session.compare),
    [notice, setNotice] = useState(""),
    [undoAvailable, setUndoAvailable] = useState(false),
    [savedViews, setSavedViews] = useState<SavedView[]>([]),
    [selectedView, setSelectedView] = useState(""),
    [refresh, setRefresh] = useState(0);
  const seq = useRef(0),
    lastSelected = useRef<string>(),
    width = { compact: 145, normal: 190, large: 260 }[zoom],
    [columns, setColumns] = useState(4),
    signature = JSON.stringify(filters) + sort + refresh;
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
    },
    saveSort = (x: GallerySort) => {
      localStorage.setItem("lumina-sort", x);
      setSort0(x);
    },
    saveListDensity = (x: ListDensity) => {
      localStorage.setItem("lumina-list-density", x);
      setListDensity0(x);
    };
  const load = useCallback(
    async (cursor?: string) => {
      const id = ++seq.current;
      setLoading(true);
      setError("");
      try {
        const page = await api.gallery(filters, cursor, 100, sort);
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
    api.savedViews().then(setSavedViews);
  }, []);
  useEffect(()=>{session.selected=[];session.compare=false},[]);
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
            ? listDensity === "compact" ? 58 : 84
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
  useEffect(() => {
    const visibleIds = visible.flatMap(item => { const row=rows[item.index]; return row?.kind === "items" ? row.items.map(asset=>asset.id) : [] });
    if (visibleIds.length) void api.prefetchThumbnails(visibleIds, 180);
    const end = visible.at(-1)?.index ?? 0;
    const nearbyIds = rows.slice(end+1,end+7).flatMap(row=>row.kind === "items" ? row.items.map(asset=>asset.id) : []);
    if (nearbyIds.length) void api.prefetchThumbnails(nearbyIds, 60);
  }, [visible.map(item=>item.index).join(), rows]);
  const toggle = (id: string, range = false) =>
      setSelection((old) => {
        const n = new Set(old);
        if (range && lastSelected.current) {
          const start = assets.findIndex(asset => asset.id === lastSelected.current);
          const end = assets.findIndex(asset => asset.id === id);
          if (start >= 0 && end >= 0) assets.slice(Math.min(start,end),Math.max(start,end)+1).forEach(asset => n.add(asset.id));
        } else n.has(id) ? n.delete(id) : n.add(id);
        lastSelected.current = id;
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
    <div className={`gallery-workspace ${preview ? "inspector-open" : ""}`}>
      <section className="gallery-canvas" aria-label="Acervo de mídias">
      <div className="gallery-command-center">
      <div className="gallery-aggregate-bar" aria-label="Resumo e filtros rápidos">
        <div className="aggregate-total"><strong>{(s?.total || 0).toLocaleString("pt-BR")} mídias</strong><span>{formatBytes(s?.bytes || 0)} no resultado atual</span></div>
        <button className={!filters.mediaType ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:undefined}))}>Todas <b>{s?.total || 0}</b></button>
        <button className={filters.mediaType === "photo" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "photo" ? undefined : "photo"}))}>Fotos <b>{s?.photos || 0}</b></button>
        <button className={filters.mediaType === "video" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "video" ? undefined : "video"}))}>Vídeos <b>{s?.videos || 0}</b></button>
        <button className={filters.mediaType === "raw" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "raw" ? undefined : "raw"}))}>RAW <b>{s?.raw || 0}</b></button>
        <button className={filters.favorite ? "active accent" : ""} onClick={()=>setFilters(value=>({...value,favorite:value.favorite ? undefined : true}))}>Favoritas <b>{s?.favorites || 0}</b></button>
        <button className={filters.protectionState === "source_only" ? "active warning" : ""} onClick={()=>setFilters(value=>({...value,protectionState:value.protectionState === "source_only" ? undefined : "source_only"}))}>Sem proteção <b>{s?.pendingProtection || 0}</b></button>
        <span className="aggregate-info">Em várias origens <b>{s?.duplicateAssets || 0}</b></span>
        <span className="aggregate-info">Metadados pendentes <b>{s?.incompleteMetadata || 0}</b></span>
      </div>
      <div className="year-strip" aria-label="Segmentar por ano">
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
        <ChoiceMenu icon={<Rows3/>} label="Ordenar" value={sort} options={[{value:"captured_desc",label:"Mais recentes"},{value:"captured_asc",label:"Mais antigas"},{value:"name_asc",label:"Nome A–Z"},{value:"name_desc",label:"Nome Z–A"},{value:"size_desc",label:"Maiores arquivos"},{value:"size_asc",label:"Menores arquivos"}]} onChange={v=>saveSort(v as GallerySort)}/>
        {mode === "grid" && (
          <ChoiceMenu icon={<Rows3/>} label="Tamanho da grade" value={zoom} options={[{value:"compact",label:"Compacta"},{value:"normal",label:"Confortável"},{value:"large",label:"Ampla"}]} onChange={v=>saveZoom(v as Zoom)}/>
        )}
        {mode === "list" && (
          <ChoiceMenu icon={<Rows3/>} label="Densidade da lista" value={listDensity} options={[{value:"comfortable",label:"Confortável"},{value:"compact",label:"Compacta"}]} onChange={v=>saveListDensity(v as ListDensity)}/>
        )}
        {savedViews.length > 0 && <select aria-label="Visões salvas" value={selectedView} onChange={e=>{setSelectedView(e.target.value);const view=savedViews.find(x=>x.id===e.target.value);if(view){setFilters(view.filters);setDraft(view.filters)}}}><option value="">Visões salvas</option>{savedViews.map(view=><option key={view.id} value={view.id}>{view.smartAlbum?"Álbum inteligente · ":""}{view.name}</option>)}</select>}
        {selectedView&&<button aria-label="Excluir visão selecionada" onClick={async()=>{await api.deleteSavedView(selectedView);setSavedViews(current=>current.filter(view=>view.id!==selectedView));setSelectedView("");setNotice("Visão removida")}}><X/> Excluir visão</button>}
        {selectedView&&<button aria-label="Renomear visão selecionada" onClick={async()=>{const current=savedViews.find(view=>view.id===selectedView);const name=prompt("Novo nome da visão",current?.name);if(name){await api.renameSavedView(selectedView,name);setSavedViews(views=>views.map(view=>view.id===selectedView?{...view,name}:view));setNotice("Visão renomeada")}}}>Renomear</button>}
        <button aria-label="Salvar visão atual" onClick={async()=>{const name=prompt("Nome da visão ou álbum inteligente");if(!name)return;const smartAlbum=confirm("Salvar também como álbum inteligente?");const view=await api.saveView(name,filters,smartAlbum);setSavedViews(v=>[...v.filter(x=>x.id!==view.id&&x.name!==view.name),view]);setNotice("Visão salva")}}><Save/> Salvar visão</button>
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
          {undoAvailable && <button onClick={async()=>{const result=await api.undoLastEdit();setNotice(result.affected?"Última alteração desfeita":"Não havia alteração para desfazer");setUndoAvailable(false);setRefresh(value=>value+1)}}>Desfazer</button>}
        </p>
      )}
      {selection.size > 0 && (
        <div className="bulk-bar">
          <strong>{selection.size} selecionadas</strong>
          <button onClick={() => setSelection(new Set(assets.map(asset=>asset.id)))}>Selecionar carregadas ({assets.length})</button>
          <button onClick={() => setAction("tag")}>Aplicar tag</button>
          <button onClick={() => setAction("album")}>Adicionar ao álbum</button>
          <button onClick={() => setAction("date")}>Corrigir data</button>
          <button onClick={async()=>{const r=await api.updateUserState({assetIds:[...selection],favorite:true});setNotice(r.affected+" favoritas");setSelection(new Set());setRefresh(v=>v+1)}}><Star/> Favoritar</button>
          <button onClick={async()=>{const r=await api.updateUserState({assetIds:[...selection],reviewLater:true});setNotice(r.affected+" marcadas para revisar");setSelection(new Set());setRefresh(v=>v+1)}}><Bookmark/> Revisar depois</button>
          {selection.size === 2 && <button className="primary" onClick={() => setComparing(true)}>Comparar</button>}
          <button onClick={() => {setSelection(new Set());lastSelected.current=undefined}}>Limpar</button>
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
        <>
        {mode === "list" && (
          <div className="gallery-list-head" aria-hidden="true">
            <span>Mídia</span><span>Captura</span><span>Arquivo</span><span>Origem</span><span>Proteção</span>
          </div>
        )}
        <div
          className={`virtual-gallery ${mode} ${mode === "list" ? `density-${listDensity}` : ""}`}
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
                       toggle={(range) => toggle(a.id, range)}
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
        </>
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
      </section>
      {preview && (
        <Preview
          asset={preview}
          position={assets.findIndex((item) => item.id === preview.id)}
          total={assets.length}
          navigate={(offset) => {
            const index = assets.findIndex((item) => item.id === preview.id);
            const next = assets[index + offset];
            if (next) setPreview(next);
          }}
          close={() => setPreview(undefined)}
          changed={(next) => {
            setPreview(next);
            setAssets((current) => {
              const updated = current.map((item) => item.id === next.id ? next : item);
              session.assets = updated;
              return updated;
            });
          }}
        />
      )}{" "}
      {action && (
        <Bulk
          action={action}
          assets={assets.filter((a) => selection.has(a.id))}
          close={() => setAction(undefined)}
          done={(x) => {
            setNotice(x);
            setUndoAvailable(true);
            setSelection(new Set());
            setAction(undefined);
            setRefresh((v) => v + 1);
          }}
        />
      )}
      {comparing && (
        <Comparison assets={assets.filter(asset=>selection.has(asset.id)).slice(0,2)} close={()=>setComparing(false)}/>
      )}
    </div>
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
  toggle: (range?: boolean) => void;
  open: () => void;
}) {
  if (mode === "list") {
    return (
      <div className={`media-card selectable list-row ${checked ? "selected" : ""}`}>
        {checked && <span className="selected-highlight"><Check /> Selecionado</span>}
        <button className="selection-check" aria-label={`Selecionar ${asset.filename}`} aria-pressed={checked} onClick={event=>toggle(event.shiftKey)}>{checked && <Check />}</button>
        <button className="media-main" aria-label={`Abrir detalhes de ${asset.filename}`} onClick={open}>
          <div className="list-media-cell">
            <MediaThumb asset={asset} />
            <span className="list-identity"><strong>{asset.filename}</strong><small>{asset.camera || "Dispositivo desconhecido"}</small></span>
          </div>
          <span className="list-capture"><time>{new Date(asset.capturedAt).toLocaleString("pt-BR")}</time>{asset.dateSuspicious && <small className="suspicious"><AlertTriangle /> Revisar data</small>}</span>
          <span className="list-file"><strong>{asset.extension.toUpperCase()}</strong><small>{formatBytes(asset.bytes)}{asset.width && asset.height ? ` · ${asset.width} × ${asset.height}` : ""}</small></span>
          <span className="list-origin"><strong>{asset.sourceNames[0] || "Acervo"}</strong><small>{asset.sourceNames.length > 1 ? `+${asset.sourceNames.length - 1} origem(ns)` : asset.mediaType === "video" ? "Vídeo" : asset.mediaType === "raw" ? "RAW" : "Foto"}</small></span>
          <span className={`list-protection ${asset.protectionState}`}><i />{asset.protectionState === "replica_verified" ? "Protegida" : asset.protectionState === "error" ? "Requer atenção" : "Pendente"}</span>
          <span className="list-markers">{asset.favorite && <Star fill="currentColor" />}{asset.reviewLater && <Bookmark />}{asset.rating > 0 && <small>{asset.rating}★</small>}</span>
        </button>
      </div>
    );
  }
  return (
    <div className={`media-card selectable ${checked ? "selected" : ""}`}>
      {checked && <span className="selected-highlight"><Check /> Selecionado</span>}
      <button
        className="selection-check"
        aria-label={`Selecionar ${asset.filename}`}
        aria-pressed={checked}
        onClick={event=>toggle(event.shiftKey)}
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
        {asset.favorite && <span className="asset-favorite" title="Favorita"><Star fill="currentColor" /></span>}
        {asset.reviewLater && <span className="asset-review"><Bookmark /> Revisar</span>}
        {asset.rating > 0 && <span className="asset-rating">{"★".repeat(asset.rating)}</span>}
        {mode === "grid" && asset.protectionState === "error" && <span className="asset-state error">Revisar proteção</span>}
      </button>
    </div>
  );
}
function Comparison({assets,close}:{assets:MediaAsset[];close:()=>void}){
  const [zoom,setZoom]=useState(1);
  useEffect(()=>{const key=(event:KeyboardEvent)=>{if(event.key==="Escape")close()};addEventListener("keydown",key);return()=>removeEventListener("keydown",key)},[close]);
  return <div className="comparison-backdrop" role="dialog" aria-modal="true" aria-label="Comparar mídias">
    <section className="comparison-shell">
      <header><div><p className="eyebrow">COMPARAÇÃO</p><h2>Lado a lado</h2><p>A comparação é visual e não altera decisões de duplicidade.</p></div><div className="comparison-tools"><button disabled={zoom===1} onClick={()=>setZoom(value=>Math.max(1,value-1))}><ZoomOut/> Reduzir</button><button disabled={zoom===3} onClick={()=>setZoom(value=>Math.min(3,value+1))}><ZoomIn/> Ampliar</button><button className="icon-only" aria-label="Fechar comparação" onClick={close}><X/></button></div></header>
      <div className="comparison-grid">{assets.map(asset=><ComparisonPane key={asset.id} asset={asset} zoom={zoom}/>)}</div>
    </section>
  </div>
}
function ComparisonPane({asset,zoom}:{asset:MediaAsset;zoom:number}){
  const [url,setUrl]=useState("");
  const [details,setDetails]=useState<AssetDetails>();
  useEffect(()=>{let live=true;const media=asset.mediaType==="video"?api.mediaUrl(asset.id):api.photoPreview(asset.id);media.then(value=>live&&setUrl(value)).catch(()=>live&&setUrl(""));api.assetDetails(asset.id).then(value=>live&&setDetails(value)).catch(()=>live&&setDetails(undefined));return()=>{live=false}},[asset.id,asset.mediaType]);
  return <article className="comparison-pane"><div className="comparison-media">{url?(asset.mediaType==="video"?<video src={url} controls preload="metadata"/>:<img src={url} alt={`Comparação de ${asset.filename}`} style={{transform:`scale(${zoom})`}}/>):<MediaThumb asset={asset}/>}</div><h3>{asset.filename}</h3><p>{new Date(asset.capturedAt).toLocaleString("pt-BR")}</p><div className="asset-pills"><span>{asset.extension.toUpperCase()}</span><span>{formatBytes(asset.bytes)}</span><span className={asset.protectionState==="replica_verified"?"success":"warning"}>{asset.protectionState==="replica_verified"?"Protegida":"Proteção pendente"}</span></div><dl><div><dt>Dimensões</dt><dd>{asset.width&&asset.height?`${asset.width} × ${asset.height}`:"Não disponível"}</dd></div><div><dt>Câmera</dt><dd>{details?.camera||asset.camera||"Não informada"}</dd></div><div><dt>Lente</dt><dd>{details?.lens||"Não informada"}</dd></div><div><dt>Captura</dt><dd>{details?.iso?`ISO ${details.iso}`:"ISO —"} · {details?.aperture?`f/${details.aperture}`:"f/—"}</dd></div><div><dt>Origens</dt><dd>{asset.sourceNames.length}</dd></div><div><dt>SHA-256</dt><dd><code>{asset.hash.slice(0,16)}…</code></dd></div></dl></article>
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
      <label>
        Organização
        <select value={value.favorite ? "favorite" : value.reviewLater ? "review" : ""} onChange={(e)=>change({...value,favorite:e.target.value==="favorite"?true:undefined,reviewLater:e.target.value==="review"?true:undefined})}>
          <option value="">Todas</option>
          <option value="favorite">Favoritas</option>
          <option value="review">Revisar depois</option>
        </select>
      </label>
      <label>
        Avaliação mínima
        <select value={value.minimumRating || ""} onChange={(e)=>change({...value,minimumRating:e.target.value?Number(e.target.value):undefined})}>
          <option value="">Qualquer</option>
          {[1,2,3,4,5].map((rating)=><option key={rating} value={rating}>{rating}+ estrelas</option>)}
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
function Preview({
  asset,
  close,
  changed,
  navigate,
  position,
  total,
}: {
  asset: MediaAsset;
  close: () => void;
  changed: (asset: MediaAsset) => void;
  navigate: (offset: -1 | 1) => void;
  position: number;
  total: number;
}) {
  const [description, setDescription] = useState(asset.description);
  const [saving, setSaving] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [previewZoom, setPreviewZoom] = useState(1);
  const [mediaUrl, setMediaUrl] = useState("");
  const [highQualityUrl, setHighQualityUrl] = useState("");
  const [qualityState, setQualityState] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [details, setDetails] = useState<AssetDetails>();
  const [detailsLoading, setDetailsLoading] = useState(false);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const drag = useRef<{ x: number; y: number; left: number; top: number }>();

  useEffect(() => {
    setDescription(asset.description);
    setPreviewZoom(1);
    setPan({ x: 0, y: 0 });
    setMediaUrl("");
    setHighQualityUrl("");
    setDetails(undefined);
    setDetailsLoading(true);
    if (asset.mediaType !== "raw") {
      api.mediaUrl(asset.id).then(setMediaUrl).catch(() => setMediaUrl(""));
    }
    if (asset.mediaType === "photo" || asset.mediaType === "raw") {
      setQualityState("loading");
      api.photoPreview(asset.id).then((url)=>{setHighQualityUrl(url);setQualityState("ready")}).catch((cause)=>{setQualityState("error");void api.recordClientError("media_error",cause instanceof Error?cause.message:String(cause))});
    } else setQualityState("idle");
    api.assetDetails(asset.id).then(setDetails).catch(() => setDetails(undefined)).finally(()=>setDetailsLoading(false));
  }, [asset.id, asset.description]);

  useEffect(() => {
    const keyboard = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
      if (event.key === "ArrowLeft") navigate(-1);
      if (event.key === "ArrowRight") navigate(1);
      if (event.key === "Escape" && fullscreen) setFullscreen(false);
    };
    window.addEventListener("keydown", keyboard);
    return () => window.removeEventListener("keydown", keyboard);
  }, [navigate, fullscreen]);

  async function update(state: Partial<Pick<MediaAsset, "favorite" | "rating" | "reviewLater" | "description">>) {
    setSaving(true);
    try {
      await api.updateUserState({ assetIds: [asset.id], ...state });
      changed({ ...asset, ...state });
    } finally {
      setSaving(false);
    }
  }

  return (
    <aside className={`drawer gallery-inspector ${fullscreen ? "fullscreen" : ""}`} aria-label="Detalhes da mídia">
      <button
        aria-label="Fechar detalhes"
        className="icon-only close"
        onClick={close}
      >
        <X />
      </button>
      <div className={`preview-stage ${previewZoom > 1 ? "pannable" : ""}`} onPointerDown={(event)=>{if(previewZoom===1)return;event.currentTarget.setPointerCapture(event.pointerId);drag.current={x:event.clientX,y:event.clientY,left:pan.x,top:pan.y}}} onPointerMove={(event)=>{if(!drag.current)return;setPan({x:drag.current.left+event.clientX-drag.current.x,y:drag.current.top+event.clientY-drag.current.y})}} onPointerUp={(event)=>{drag.current=undefined;event.currentTarget.releasePointerCapture(event.pointerId)}}>
        {asset.mediaType === "video" && mediaUrl ? (
          <video key={asset.id} className="drawer-video" src={mediaUrl} controls preload="metadata" />
        ) : (asset.mediaType === "photo" || asset.mediaType === "raw") && (highQualityUrl || mediaUrl) ? (
          <img key={`${asset.id}-${highQualityUrl ? "hq" : "fast"}`} className="drawer-photo" src={highQualityUrl || mediaUrl} alt={`Prévia de ${asset.filename}`} draggable={false} style={{transform:`translate(${pan.x}px,${pan.y}px) scale(${previewZoom})`}} />
        ) : (
          <MediaThumb key={asset.id} asset={asset} className={`drawer-preview preview-zoom-${previewZoom}`} />
        )}
        <div className="preview-tools">
          <button aria-label="Diminuir zoom" disabled={previewZoom === 1} onClick={() => setPreviewZoom((value) => Math.max(1, value - 1))}><ZoomOut /></button>
          <button aria-label="Aumentar zoom" disabled={previewZoom === 3} onClick={() => setPreviewZoom((value) => Math.min(3, value + 1))}><ZoomIn /></button>
          <button aria-label={fullscreen ? "Sair da tela cheia" : "Abrir em tela cheia"} onClick={() => setFullscreen((value) => !value)}>{fullscreen ? <Minimize2 /> : <Maximize2 />}</button>
        </div>
        {qualityState === "loading" && <span className="preview-quality"><LoaderCircle className="spin"/> Preparando alta qualidade</span>}
        {qualityState === "ready" && <span className="preview-quality ready">Prévia HD</span>}
      </div>
      <div className="preview-navigation">
        <button
          aria-label="Mídia anterior"
          disabled={position <= 0}
          onClick={() => navigate(-1)}
        >
          <ChevronLeft />
        </button>
        <span>{position + 1} de {total}</span>
        <button
          aria-label="Próxima mídia"
          disabled={position < 0 || position >= total - 1}
          onClick={() => navigate(1)}
        >
          <ChevronRight />
        </button>
      </div>
      <h2>{asset.filename}</h2>
      <p>{new Date(asset.capturedAt).toLocaleString("pt-BR")}</p>
      <div className="asset-pills" aria-label="Atributos da mídia">
        <span>{asset.mediaType === "video" ? "Vídeo" : asset.mediaType === "raw" ? "RAW" : "Foto"}</span>
        <span>{asset.extension.toUpperCase()}</span>
        {asset.favorite && <span className="accent">Favorita</span>}
        <span className={asset.protectionState === "replica_verified" ? "success" : "warning"}>{asset.protectionState === "replica_verified" ? "Protegida" : "Proteção pendente"}</span>
        {asset.tags.map(tag=><span key={tag}>#{tag}</span>)}
      </div>
      <div className="asset-personal-actions" aria-label="Organização pessoal">
        <button
          className={asset.favorite ? "active" : ""}
          aria-pressed={asset.favorite}
          disabled={saving}
          onClick={() => update({ favorite: !asset.favorite })}
        >
          <Star fill={asset.favorite ? "currentColor" : "none"} /> Favorita
        </button>
        <button
          className={asset.reviewLater ? "active" : ""}
          aria-pressed={asset.reviewLater}
          disabled={saving}
          onClick={() => update({ reviewLater: !asset.reviewLater })}
        >
          <Bookmark fill={asset.reviewLater ? "currentColor" : "none"} /> Revisar
        </button>
      </div>
      {asset.reviewLater && (
        <button className="complete-review" disabled={saving} onClick={async () => { await update({ reviewLater: false }); if (position < total - 1) navigate(1); }}>
          <Check /> Concluir revisão e avançar
        </button>
      )}
      <div className="asset-stars" aria-label="Avaliação">
        {[1, 2, 3, 4, 5].map((rating) => (
          <button
            key={rating}
            aria-label={`${rating} estrelas`}
            aria-pressed={asset.rating === rating}
            disabled={saving}
            onClick={() => update({ rating: asset.rating === rating ? 0 : rating })}
          >
            <Star fill={rating <= asset.rating ? "currentColor" : "none"} />
          </button>
        ))}
      </div>
      <label className="asset-description">
        Descrição
        <textarea
          value={description}
          maxLength={2000}
          placeholder="Contexto, pessoas, ocasião ou lembrete…"
          onChange={(event) => setDescription(event.target.value)}
          onBlur={() => {
            if (description !== asset.description) update({ description });
          }}
        />
      </label>
      {asset.dateSuspicious && (
        <p className="notice warning">
          <AlertTriangle /> Data a revisar
        </p>
      )}
      <MetadataSection id="capture" title="Captura" openByDefault>
      {detailsLoading && <p className="metadata-state"><LoaderCircle className="spin"/> Lendo metadados do arquivo…</p>}
      <Info label="Câmera" value={details?.camera || asset.camera || "Não informado no arquivo"} />
      <Info label="Lente" value={details?.lens || "Não disponível"} />
      <Info label="Exposição" value={formatExposure(details?.exposure)} />
      <Info label="Abertura" value={details?.aperture ? `f/${details.aperture}` : "Não disponível"} />
      <Info label="ISO" value={details?.iso?.toString() || "Não disponível"} />
      <Info label="Distância focal" value={details?.focalLength ? `${details.focalLength} mm` : "Não disponível"} />
      </MetadataSection>
      <MetadataSection id="file" title="Arquivo e mídia" openByDefault>
      <Info
        label="Tipo"
        value={`${asset.mediaType} · ${(details?.detectedFormat || asset.extension).toUpperCase()}`}
      />
      <Info label="Tamanho" value={formatBytes(asset.bytes)} />
      <Info label="Dimensões" value={asset.width && asset.height ? `${asset.width} × ${asset.height} px` : "Não disponível"} />
      <Info label="Resolução" value={asset.width&&asset.height?`${(asset.width*asset.height/1_000_000).toFixed(1)} MP`:"Não disponível"}/>
      {asset.duration!=null&&<Info label="Duração" value={formatDuration(asset.duration)} />}
      {asset.mediaType==="video"&&<><Info label="Contêiner" value={details?.container || "Não disponível"}/><Info label="Codec de vídeo" value={details?.codec || "Não disponível"}/><Info label="Quadros por segundo" value={details?.frameRate ? `${details.frameRate.toFixed(2)} fps` : "Não disponível"}/><Info label="Codec de áudio" value={details?.audioCodec || "Não disponível"}/><Info label="Taxa de bits" value={details?.bitrate ? `${(details.bitrate/1_000_000).toFixed(2)} Mb/s` : "Não disponível"}/></>}
      {details?.inventoryError&&<p className="notice warning"><AlertTriangle/>Metadados incompletos: {details.inventoryError}</p>}
      </MetadataSection>
      <MetadataSection id="locations" title="Localizações" openByDefault>
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
      <button className="reveal-file" onClick={async()=>{try{await api.revealAsset(asset.id)}catch(error){void api.recordClientError("reveal_error",String(error))}}}><HardDrive/> Abrir localização no Explorador</button>
      </MetadataSection>
      <MetadataSection id="catalog" title="Catálogo">
      <p className="hash">
        SHA-256
        <br />
        <code>{asset.hash}</code>
      </p>
      <button className="copy-value" onClick={()=>navigator.clipboard.writeText(asset.hash)}><Copy/> Copiar SHA-256</button>
      </MetadataSection>
    </aside>
  );
}
function MetadataSection({id,title,openByDefault=false,children}:{id:string;title:string;openByDefault?:boolean;children:React.ReactNode}){
  const key=`lumina-metadata-${id}`;
  const [open,setOpen]=useState(()=>localStorage.getItem(key)?.toString()==="open"||(localStorage.getItem(key)===null&&openByDefault));
  return <details className="metadata-section" open={open} onToggle={event=>{const value=event.currentTarget.open;setOpen(value);localStorage.setItem(key,value?"open":"closed")}}><summary><span>{title}</span><ChevronDown/></summary><div>{children}</div></details>
}
function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
function formatDuration(seconds:number){const rounded=Math.round(seconds);return `${Math.floor(rounded/60)}:${String(rounded%60).padStart(2,"0")}`}
function formatExposure(value?:string){if(!value)return "Não disponível";const number=Number(value);if(Number.isFinite(number)&&number>0&&number<1)return `1/${Math.round(1/number)} s`;return `${value}${value.includes("s")?"":" s"}`}
