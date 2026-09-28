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
  MapPin,
  MapPinned,
  Rows3,
  Search,
  Star,
  Bookmark,
  Save,
  Tags,
  Video,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  X,
} from "lucide-react";
import { api } from "./api";
import { captureDate, formatBytes, formatCaptureDate } from "./format";
import { BoundedLru } from "./lru";
import MediaViewer from "./MediaViewer";
import type { Album, AssetDetails, CurationPage, CurationSession, GalleryFilters, GalleryResult, GallerySort, MediaAsset, SavedView } from "./types";
const thumbs = new BoundedLru<string, string | null>(512),
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
const captureSourceLabel = (source: string) => source === "exif_original_offset"
  ? "Câmera · fuso informado"
  : source.startsWith("exif_original")
    ? "Câmera · horário local"
    : source === "media_created"
      ? "Criação da mídia"
      : source === "user_corrected"
        ? "Data corrigida"
        : "Data do arquivo";
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
  const ids=assetIds.slice(0,4);
  session.filters = { ...empty, assetIds:ids };
  session.result = undefined;
  session.assets = [];
  session.scrollY = 0;
  session.selected = ids;
  session.compare = session.selected.length >= 2 && session.selected.length <= 4;
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
    [action, setAction] = useState<"tag" | "album" | "date" | "location">(),
    [comparing, setComparing] = useState(()=>session.compare),
    [notice, setNotice] = useState(""),
    [undoAvailable, setUndoAvailable] = useState(false),
    [savedViews, setSavedViews] = useState<SavedView[]>([]),
    [selectedView, setSelectedView] = useState(""),
    [viewDialog,setViewDialog]=useState<"save"|"rename">(),
    [curationOpen,setCurationOpen]=useState(false),
    [curationSessions,setCurationSessions]=useState<CurationSession[]>([]),
    [curationPage,setCurationPage]=useState<CurationPage>(),
    [inspectorWidth,setInspectorWidth]=useState(()=>Number(localStorage.getItem("lumina-inspector-width"))||410),
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
    api.curationSessions().then(setCurationSessions);
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
      const d = captureDate(a.capturedAt),
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
  const applyUserState=useCallback(async(patch:{favorite?:boolean;rating?:number;reviewLater?:boolean},label:string)=>{
    const ids=[...selection];
    if(!ids.length)return;
    try{
      const changed=await api.updateUserState({assetIds:ids,...patch});
      setNotice(`${changed.affected} mídias · ${label}`);
      setUndoAvailable(changed.affected>0);
      setSelection(new Set());
      setRefresh(value=>value+1);
    }catch(cause){setNotice(String(cause))}
  },[selection]);
  const resumeCuration=useCallback(async(id:string)=>{
    try{
      const page=await api.curationPage(id);
      setCurationPage(page);
      setCurationSessions(current=>current.map(item=>item.id===id?page.session:item));
      setFilters({...empty,assetIds:page.assetIds});
      setDraft({...empty,assetIds:page.assetIds});
      saveSort(page.session.sort);
      setSelection(new Set());
      setCurationOpen(false);
      setNotice(page.remaining?`Curadoria “${page.session.name}” retomada`:`Curadoria “${page.session.name}” concluída`);
    }catch(cause){setNotice(String(cause))}
  },[]);
  const decideCuration=useCallback(async(decision:"reviewed"|"skipped")=>{
    if(!curationPage||!selection.size)return;
    try{
      const ids=[...selection];
      const updated=await api.updateCuration(curationPage.session.id,ids,decision);
      const page=await api.curationPage(updated.id);
      setCurationPage(page);
      setCurationSessions(current=>current.map(item=>item.id===updated.id?updated:item));
      setFilters({...empty,assetIds:page.assetIds});
      setSelection(new Set());
      setNotice(decision==="reviewed"?`${ids.length} mídias revisadas`:`${ids.length} mídias puladas`);
    }catch(cause){setNotice(String(cause))}
  },[curationPage,selection]);
  useEffect(()=>{
    const onKey=(event:KeyboardEvent)=>{
      const target=event.target;
      if((target instanceof Element&&target.closest("input,textarea,select,[contenteditable=true]"))||!selection.size)return;
      if(event.key==="Escape"){setSelection(new Set());return}
      if(event.key>="0"&&event.key<="5"){event.preventDefault();void applyUserState({rating:Number(event.key)},`avaliação ${event.key}`);return}
      if(event.key.toLowerCase()==="f"){event.preventDefault();void applyUserState({favorite:!event.shiftKey},event.shiftKey?"removidas das favoritas":"favoritadas");return}
      if(event.key.toLowerCase()==="r"){event.preventDefault();void applyUserState({reviewLater:!event.shiftKey},event.shiftKey?"revisão concluída":"revisar depois");}
    };
    addEventListener("keydown",onKey);
    return()=>removeEventListener("keydown",onKey);
  },[selection,applyUserState]);
  return (
    <div className={`gallery-workspace ${preview ? "inspector-open" : ""}`} style={{"--inspector-width":`${inspectorWidth}px`} as React.CSSProperties}>
      <section className="gallery-canvas" aria-label="Acervo de mídias">
      <div className="gallery-command-center">
      <div className="gallery-aggregate-bar" aria-label="Resumo e filtros rápidos">
        <div className="aggregate-total"><strong>{(s?.total || 0).toLocaleString("pt-BR")} mídias</strong><span>{formatBytes(s?.bytes || 0)} no resultado atual</span></div>
        <button className={!filters.mediaType ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:undefined}))}>Todas <b>{s?.total || 0}</b></button>
        <button className={filters.mediaType === "photo" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "photo" ? undefined : "photo"}))}>Fotos <b>{s?.photos || 0}</b></button>
        <button className={filters.mediaType === "video" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "video" ? undefined : "video"}))}>Vídeos <b>{s?.videos || 0}</b></button>
        <button className={filters.mediaType === "raw" ? "active" : ""} onClick={()=>setFilters(value=>({...value,mediaType:value.mediaType === "raw" ? undefined : "raw"}))}>RAW <b>{s?.raw || 0}</b></button>
        <button className={filters.favorite ? "active accent" : ""} onClick={()=>setFilters(value=>({...value,favorite:value.favorite ? undefined : true}))}>Favoritas <b>{s?.favorites || 0}</b></button>
        <button className={filters.hasLocation ? "active" : ""} onClick={()=>setFilters(value=>({...value,hasLocation:value.hasLocation ? undefined : true}))}>Com localização <b>{s?.withLocation || 0}</b></button>
        <button className={filters.reviewLater ? "active warning" : ""} onClick={()=>setFilters(value=>({...value,reviewLater:value.reviewLater ? undefined : true}))}>Revisar depois</button>
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
        {selectedView&&<button aria-label="Renomear visão selecionada" onClick={()=>setViewDialog("rename")}>Renomear</button>}
        <button aria-label="Salvar visão atual" onClick={()=>setViewDialog("save")}><Save/> Salvar visão</button>
        <button aria-label="Abrir sessões de curadoria" onClick={()=>setCurationOpen(true)}><Bookmark/> Curadoria {curationSessions.filter(item=>item.state==="active").length>0&&<b>{curationSessions.filter(item=>item.state==="active").length}</b>}</button>
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
      {curationPage&&<div className="curation-strip">
        <div><span className="eyebrow">CURADORIA RETOMÁVEL</span><strong>{curationPage.session.name}</strong><small>{curationPage.session.reviewedItems+curationPage.session.skippedItems} de {curationPage.session.totalItems} decididas · {curationPage.remaining} restantes</small></div>
        <div className="curation-progress"><i style={{width:`${curationPage.session.totalItems?((curationPage.session.reviewedItems+curationPage.session.skippedItems)/curationPage.session.totalItems)*100:100}%`}}/></div>
        <button onClick={()=>{setFilters(curationPage.session.filters);setDraft(curationPage.session.filters);setCurationPage(undefined);setSelection(new Set())}}>Sair da sessão</button>
      </div>}
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
          <button onClick={() => setAction("location")}><MapPinned/> Nomear lugar</button>
          <button onClick={()=>void applyUserState({favorite:true},"favoritadas")}><Star/> Favoritar <kbd>F</kbd></button>
          <button onClick={()=>void applyUserState({favorite:false},"removidas das favoritas")}>Remover favorita <kbd>⇧F</kbd></button>
          <button onClick={()=>void applyUserState({reviewLater:true},"revisar depois")}><Bookmark/> Revisar depois <kbd>R</kbd></button>
          <button onClick={()=>void applyUserState({reviewLater:false},"revisão concluída")}>Concluir revisão <kbd>⇧R</kbd></button>
          <ChoiceMenu icon={<Star/>} label="Avaliação" value="" options={[1,2,3,4,5].map(value=>({value:String(value),label:`${value} estrela${value>1?"s":""}`}))} onChange={value=>void applyUserState({rating:Number(value)},`avaliação ${value}`)}/>
          {curationPage&&<><button className="primary" onClick={()=>void decideCuration("reviewed")}><Check/> Marcar revisadas</button><button onClick={()=>void decideCuration("skipped")}>Pular nesta sessão</button></>}
          {selection.size >= 2 && selection.size <= 4 && <button className="primary" onClick={() => setComparing(true)}>Comparar {selection.size}</button>}
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
      {preview && <div className="inspector-resizer" role="separator" aria-label="Redimensionar painel de inspeção" aria-orientation="vertical" tabIndex={0} onKeyDown={event=>{if(!["ArrowLeft","ArrowRight"].includes(event.key))return;const next=Math.min(620,Math.max(340,inspectorWidth+(event.key==="ArrowLeft"?16:-16)));setInspectorWidth(next);localStorage.setItem("lumina-inspector-width",String(next))}} onPointerDown={event=>event.currentTarget.setPointerCapture(event.pointerId)} onPointerMove={event=>{if(!event.currentTarget.hasPointerCapture(event.pointerId))return;setInspectorWidth(Math.min(620,Math.max(340,window.innerWidth-event.clientX-20)))}} onPointerUp={event=>{event.currentTarget.releasePointerCapture(event.pointerId);localStorage.setItem("lumina-inspector-width",String(inspectorWidth))}} />}
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
      {viewDialog&&<CollectionDialog mode={viewDialog} initialName={viewDialog==="rename"?savedViews.find(view=>view.id===selectedView)?.name||"":""} close={()=>setViewDialog(undefined)} submit={async(name,smart)=>{if(viewDialog==="rename"){await api.renameSavedView(selectedView,name);setSavedViews(views=>views.map(view=>view.id===selectedView?{...view,name}:view));setNotice("Visão renomeada")}else{const view=await api.saveView(name,filters,smart);setSavedViews(value=>[...value.filter(item=>item.id!==view.id&&item.name!==view.name),view]);setNotice(smart?"Álbum inteligente salvo":"Visão salva")}setViewDialog(undefined)}}/>}
      {curationOpen&&<CurationManager sessions={curationSessions} filters={filters} sort={sort} close={()=>setCurationOpen(false)} resume={resumeCuration} changed={setCurationSessions} notice={setNotice}/>}
      {comparing && assets.filter(asset=>selection.has(asset.id)).length >= 2 && (
        <Comparison assets={assets.filter(asset=>selection.has(asset.id)).slice(0,4)} close={()=>setComparing(false)} done={message=>{setNotice(message);setUndoAvailable(true);setRefresh(value=>value+1)}}/>
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
          <span className="list-capture"><time>{formatCaptureDate(asset.capturedAt)}</time>{asset.dateSuspicious && <small className="suspicious"><AlertTriangle /> Revisar data</small>}</span>
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
function CollectionDialog({mode,initialName,close,submit}:{mode:"save"|"rename";initialName:string;close:()=>void;submit:(name:string,smart:boolean)=>Promise<void>}){
  const[name,setName]=useState(initialName),[smart,setSmart]=useState(true),[busy,setBusy]=useState(false),[error,setError]=useState("");
  return <div className="modal-backdrop" role="dialog" aria-modal="true" aria-label={mode==="save"?"Salvar visão":"Renomear visão"}><form className="modal compact" onSubmit={async event=>{event.preventDefault();if(!name.trim())return;setBusy(true);setError("");try{await submit(name.trim(),smart)}catch(cause){setError(String(cause));setBusy(false)}}}><button type="button" className="icon-only close" onClick={close}><X/></button><p className="eyebrow">COLEÇÃO DINÂMICA</p><h2>{mode==="save"?"Salvar consulta atual":"Renomear consulta"}</h2><p>Os filtros são reavaliados sempre que o acervo muda; nenhum arquivo é movido.</p><label>Nome<input autoFocus maxLength={100} value={name} onChange={event=>setName(event.target.value)}/></label>{mode==="save"&&<label className="check-line"><input type="checkbox" checked={smart} onChange={event=>setSmart(event.target.checked)}/> Exibir como álbum inteligente</label>}{error&&<p role="alert" className="error-text">{error}</p>}<div className="modal-actions"><button type="button" onClick={close}>Cancelar</button><button className="primary" disabled={busy||!name.trim()}>{busy?"Salvando…":"Salvar"}</button></div></form></div>
}
function CurationManager({sessions,filters,sort,close,resume,changed,notice}:{sessions:CurationSession[];filters:GalleryFilters;sort:GallerySort;close:()=>void;resume:(id:string)=>Promise<void>;changed:(value:CurationSession[])=>void;notice:(value:string)=>void}){
  const[name,setName]=useState(""),[busy,setBusy]=useState(false),[error,setError]=useState(""),[confirmDelete,setConfirmDelete]=useState("");
  const create=async(event:React.FormEvent)=>{event.preventDefault();if(!name.trim())return;setBusy(true);setError("");try{const created=await api.createCuration(name.trim(),filters,sort);changed([created,...sessions]);await resume(created.id)}catch(cause){setError(String(cause));setBusy(false)}};
  return <div className="modal-backdrop" role="dialog" aria-modal="true" aria-label="Sessões de curadoria"><section className="modal curation-manager"><button className="icon-only close" onClick={close}><X/></button><p className="eyebrow">CURADORIA RETOMÁVEL</p><h2>Revisar o acervo sem perder o ponto</h2><p>A sessão congela a ordem dos resultados atuais. Favoritos, notas e tags continuam no catálogo, mas os arquivos físicos não mudam.</p><form className="curation-create" onSubmit={create}><label>Nome da nova sessão<input value={name} maxLength={100} onChange={event=>setName(event.target.value)} placeholder="Ex.: Seleção de férias 2025"/></label><button className="primary" disabled={busy||!name.trim()}>{busy?"Criando…":`Criar com ${filters.assetIds?.length||"todos os"} resultados`}</button></form>{error&&<p role="alert" className="error-text">{error}</p>}<div className="curation-session-list">{sessions.length?sessions.map(item=>{const decided=item.reviewedItems+item.skippedItems,percent=item.totalItems?Math.round(decided/item.totalItems*100):100;return <article key={item.id}><div><strong>{item.name}</strong><span className={`status-pill ${item.state}`}>{item.state==="completed"?"Concluída":"Em andamento"}</span><small>{decided} de {item.totalItems} · {percent}%</small><div className="curation-progress"><i style={{width:`${percent}%`}}/></div></div><div><button disabled={item.state==="completed"} onClick={()=>void resume(item.id)}>{item.reviewedItems||item.skippedItems?"Retomar":"Começar"}</button>{confirmDelete===item.id?<><button className="subtle-danger" onClick={async()=>{await api.deleteCuration(item.id);changed(sessions.filter(session=>session.id!==item.id));setConfirmDelete("");notice("Sessão removida; decisões das mídias foram preservadas")}}>Confirmar exclusão</button><button onClick={()=>setConfirmDelete("")}>Cancelar</button></>:<button onClick={()=>setConfirmDelete(item.id)}>Excluir sessão</button>}</div></article>}):<p className="empty-state">Nenhuma sessão criada.</p>}</div></section></div>
}
function Comparison({assets,close,done}:{assets:MediaAsset[];close:()=>void;done:(message:string)=>void}){
  const [zoom,setZoom]=useState(1),[synced,setSynced]=useState(true),[pan,setPan]=useState<[number,number]>([50,50]),[perZoom,setPerZoom]=useState<Record<string,number>>({}),[details,setDetails]=useState<Record<string,AssetDetails>>({}),[winner,setWinner]=useState("");
  useEffect(()=>{let live=true;Promise.all(assets.map(async asset=>[asset.id,await api.assetDetails(asset.id)] as const)).then(entries=>{if(live)setDetails(Object.fromEntries(entries))}).catch(()=>{});return()=>{live=false}},[assets.map(asset=>asset.id).join()]);
  useEffect(()=>{const key=(event:KeyboardEvent)=>{if(event.key==="Escape")close()};addEventListener("keydown",key);return()=>removeEventListener("keydown",key)},[close]);
  const values=(asset:MediaAsset)=>({dimensions:asset.width&&asset.height?`${asset.width}×${asset.height}`:"",camera:details[asset.id]?.camera||asset.camera||"",lens:details[asset.id]?.lens||"",capture:`${details[asset.id]?.iso||""}/${details[asset.id]?.aperture||""}`,bytes:String(asset.bytes),date:asset.capturedAt,origins:String(asset.sourceNames.length)}),different=new Set(Object.keys(values(assets[0])).filter(key=>new Set(assets.map(asset=>values(asset)[key as keyof ReturnType<typeof values>])).size>1));
  const choose=async(asset:MediaAsset)=>{try{await api.chooseComparisonWinner(asset.id,assets.map(item=>item.id));setWinner(asset.id);done(`${asset.filename} escolhida; alternativas ficaram em “Revisar depois”`)}catch(cause){done(String(cause))}};
  return <div className="comparison-backdrop" role="dialog" aria-modal="true" aria-label="Comparar mídias">
    <section className="comparison-shell">
      <header><div><p className="eyebrow">COMPARAÇÃO · {assets.length} MÍDIAS</p><h2>Lado a lado</h2><p>Diferenças estão destacadas. A escolha é reversível e não remove nenhum arquivo.</p></div><div className="comparison-tools"><label className="sync-toggle"><input type="checkbox" checked={synced} onChange={event=>setSynced(event.target.checked)}/> Zoom sincronizado</label>{synced&&<><button disabled={zoom<=1} onClick={()=>setZoom(value=>Math.max(1,value-.25))}><ZoomOut/> Reduzir</button><button disabled={zoom>=4} onClick={()=>setZoom(value=>Math.min(4,value+.25))}><ZoomIn/> Ampliar</button></>}<button className="icon-only" aria-label="Fechar comparação" onClick={close}><X/></button></div></header>
      {synced&&zoom>1&&<div className="comparison-framing"><label>Enquadramento horizontal <input aria-label="Enquadramento horizontal" type="range" min="0" max="100" value={pan[0]} onChange={event=>setPan([Number(event.target.value),pan[1]])}/></label><label>Vertical <input aria-label="Enquadramento vertical" type="range" min="0" max="100" value={pan[1]} onChange={event=>setPan([pan[0],Number(event.target.value)])}/></label><button onClick={()=>{setZoom(1);setPan([50,50])}}><RotateCcw/> Redefinir</button></div>}
      <div className="comparison-grid">{assets.map(asset=><ComparisonPane key={asset.id} asset={asset} details={details[asset.id]} different={different} zoom={synced?zoom:perZoom[asset.id]||1} pan={synced?pan:[50,50]} synced={synced} winner={winner===asset.id} choose={()=>void choose(asset)} changeZoom={value=>setPerZoom(current=>({...current,[asset.id]:value}))}/>)}</div>
    </section>
  </div>
}
function ComparisonPane({asset,details,different,zoom,pan,synced,winner,choose,changeZoom}:{asset:MediaAsset;details?:AssetDetails;different:Set<string>;zoom:number;pan:[number,number];synced:boolean;winner:boolean;choose:()=>void;changeZoom:(value:number)=>void}){
  const [url,setUrl]=useState("");
  useEffect(()=>{let live=true;const media=asset.mediaType==="video"?api.mediaUrl(asset.id):api.photoPreview(asset.id);media.then(value=>live&&setUrl(value)).catch(()=>live&&setUrl(""));return()=>{live=false}},[asset.id,asset.mediaType]);
  const row=(key:string,label:string,value:React.ReactNode)=><div className={different.has(key)?"comparison-difference":""}><dt>{label}{different.has(key)&&<span>difere</span>}</dt><dd>{value}</dd></div>;
  return <article className={`comparison-pane ${winner?"winner":""}`}><div className="comparison-media"><div style={{transform:`scale(${zoom})`,transformOrigin:`${pan[0]}% ${pan[1]}%`}}>{url?(asset.mediaType==="video"?<ManagedVideo key={url} src={url} className="comparison-video"/>:<img src={url} alt={`Comparação de ${asset.filename}`}/>):<MediaThumb asset={asset}/>}</div></div>{!synced&&<div className="pane-zoom"><button aria-label={`Reduzir ${asset.filename}`} disabled={zoom<=1} onClick={()=>changeZoom(Math.max(1,zoom-.25))}><ZoomOut/></button><span>{Math.round(zoom*100)}%</span><button aria-label={`Ampliar ${asset.filename}`} disabled={zoom>=4} onClick={()=>changeZoom(Math.min(4,zoom+.25))}><ZoomIn/></button></div>}<h3>{asset.filename}</h3><p>{formatCaptureDate(asset.capturedAt)}</p><div className="asset-pills"><span>{asset.extension.toUpperCase()}</span><span className={different.has("bytes")?"different":""}>{formatBytes(asset.bytes)}</span><span className={asset.protectionState==="replica_verified"?"success":"warning"}>{asset.protectionState==="replica_verified"?"Protegida":"Proteção pendente"}</span></div><dl>{row("dimensions","Dimensões",asset.width&&asset.height?`${asset.width} × ${asset.height}`:"Não disponível")}{row("camera","Câmera",details?.camera||asset.camera||"Não informada")}{row("lens","Lente",details?.lens||"Não informada")}{row("capture","Captura",<>{details?.iso?`ISO ${details.iso}`:"ISO —"} · {details?.aperture?`f/${details.aperture}`:"f/—"}</>)}{row("origins","Origens",asset.sourceNames.length)}<div><dt>SHA-256</dt><dd><code>{asset.hash.slice(0,16)}…</code></dd></div></dl><button className="primary comparison-winner" disabled={winner} onClick={choose}>{winner?<><Check/> Escolhida</>:"Escolher esta"}</button></article>
}

function ManagedVideo({ src, className }: { src: string; className?: string }) {
  const ref = useRef<HTMLVideoElement>(null);
  useEffect(() => {
    const video = ref.current;
    return () => {
      if (!video) return;
      video.pause();
      video.removeAttribute("src");
      video.load();
    };
  }, []);
  return <video ref={ref} src={src} className={className} controls preload="metadata" />;
}
function Bulk({
  action,
  assets,
  close,
  done,
}: {
  action: "tag" | "album" | "date" | "location";
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
              : action === "location"
                ? await api.renameAssetsLocation(ids,value)
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
              : action === "location"
                ? "Nomear lugar nas selecionadas"
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
            aria-label={action === "tag" ? "Nome da tag" : action === "location" ? "Nome do lugar" : "Nova data"}
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
  const [details, setDetails] = useState<AssetDetails>();
  const [detailsLoading, setDetailsLoading] = useState(false);
  const [fileAction, setFileAction] = useState("");
  useEffect(() => {
    let live=true;
    setDescription(asset.description);setDetails(undefined);setDetailsLoading(true);setFileAction("");
    const timer=setTimeout(()=>{api.assetDetails(asset.id).then(value=>{if(live)setDetails(value)}).catch(()=>{}).finally(()=>{if(live)setDetailsLoading(false)})},180);
    return()=>{live=false;clearTimeout(timer)};
  },[asset.id]);
  useEffect(()=>{
    const keyboard=(event:KeyboardEvent)=>{
      if((event.target as HTMLElement)?.closest("input,textarea,select,[contenteditable=true],[role=separator]"))return;
      if(event.key==="ArrowLeft"){event.preventDefault();navigate(-1)}
      if(event.key==="ArrowRight"){event.preventDefault();navigate(1)}
      if(event.key==="Escape"){event.preventDefault();if(fullscreen)setFullscreen(false);else close()}
    };
    window.addEventListener("keydown",keyboard);return()=>window.removeEventListener("keydown",keyboard);
  },[navigate,fullscreen,close]);

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
      <div className="inspector-heading"><strong title={asset.filename}>{asset.filename}</strong><button aria-label="Fechar detalhes" className="icon-only close" onClick={close}><X/></button></div>
      <MediaViewer key={asset.id} asset={asset} fullscreen={fullscreen} toggleFullscreen={()=>setFullscreen(value=>!value)}/>
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
      <div className="inspector-details">
      <h2>{asset.filename}</h2>
      <p>{formatCaptureDate(asset.capturedAt)}</p>
      <div className="asset-pills" aria-label="Atributos da mídia">
        <span>{asset.mediaType === "video" ? "Vídeo" : asset.mediaType === "raw" ? "RAW" : "Foto"}</span>
        <span>{asset.extension.toUpperCase()}</span>
        <span title="Origem usada para ordenar a linha do tempo">{captureSourceLabel(asset.dateSource)}</span>
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
      {asset.latitude != null && asset.longitude != null && <MetadataSection id="geography" title="Localização geográfica" openByDefault>
        <div className="location good">
          <MapPin />
          <div><strong>{details?.placeName || "Coordenadas disponíveis"}</strong><small>{[details?.sublocation,details?.locationCity,details?.locationRegion,details?.locationCountry].filter((value,index,all)=>value&&all.indexOf(value)===index).join(" · ") || `${asset.latitude.toFixed(6)}, ${asset.longitude.toFixed(6)}`}</small></div>
        </div>
        <div className="asset-pills">
          <span>{details?.locationSource==="embedded"?"Do arquivo":details?.locationSource==="offline"?"Base offline":"Aproximada"}</span>
          <span>{details?.locationConfidence==="exact"?"Confiança exata":details?.locationConfidence==="probable"?"Confiança provável":"Confiança aproximada"}</span>
          {details?.locationAccuracyM!=null&&<span>Precisão ±{Math.round(details.locationAccuracyM)} m</span>}
          {details?.altitude!=null&&<span>Altitude {Math.round(details.altitude)} m</span>}
        </div>
        <button className="copy-value" onClick={async()=>{await navigator.clipboard.writeText(`${asset.latitude}, ${asset.longitude}`);setFileAction("Coordenadas copiadas.")}}><Copy/> Copiar coordenadas</button>
      </MetadataSection>}
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
      <button className="reveal-file" onClick={async()=>{setFileAction("");try{const result=await api.revealAsset(asset.id);setFileAction(result==="selected"?"Arquivo selecionado no Explorador.":"O Explorer abriu a pasta, mas o Windows não selecionou o arquivo.")}catch(error){const message=String(error);setFileAction(message);void api.recordClientError("reveal_error",message)}}}><HardDrive/> Mostrar arquivo no Explorador</button>
      {fileAction&&<p className={fileAction.includes("selecionado")||fileAction.includes("copiadas")?"metadata-action success":"metadata-action error"} role="status">{fileAction}</p>}
      </MetadataSection>
      <MetadataSection id="catalog" title="Catálogo">
      <p className="hash">
        SHA-256
        <br />
        <code>{asset.hash}</code>
      </p>
      <button className="copy-value" onClick={()=>navigator.clipboard.writeText(asset.hash)}><Copy/> Copiar SHA-256</button>
      </MetadataSection>
      </div>
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
