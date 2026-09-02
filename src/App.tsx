import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Activity,
  Album as AlbumIcon,
  Archive,
  ClipboardCheck,
  CheckCircle2,
  ChevronRight,
  Cloud,
  Copy,
  Database,
  Folder,
  FolderOpen,
  HardDrive,
  Images,
  LayoutDashboard,
  LoaderCircle,
  Plus,
  RefreshCw,
  Search,
  ShieldCheck,
  X,
} from "lucide-react";
import { api } from "./api";
import Gallery, { MediaThumb, openGalleryWithFilters } from "./Gallery";
import ActivityCenter from "./ActivityCenter";
import "./gallery.css";
import { formatBytes, formatDate } from "./format";
import type {
  Album,
  CleanupPlan,
  DashboardStats,
  DuplicateGroup,
  DuplicateStatus,
  ImportEvent,
  ImportSummary,
  JobOverview,
  JobProgress,
  LibraryConfig,
  LibraryHealth,
  RecoverableJob,
  Source,
  SavedView,
  TagInfo,
  StoragePlan,
  ThumbnailAudit,
  ThumbnailRepairProgress,
  ReviewSummary,
  View,
} from "./types";
const nav = [
  { id: "dashboard", label: "Visão geral", icon: LayoutDashboard },
  { id: "library", label: "Biblioteca", icon: Images },
  { id: "review", label: "Revisão", icon: ClipboardCheck },
  { id: "sources", label: "Fontes", icon: HardDrive },
  { id: "duplicates", label: "Duplicatas", icon: Copy },
  { id: "albums", label: "Álbuns", icon: AlbumIcon },
  { id: "activity", label: "Atividade", icon: Activity },
  { id: "protection", label: "Proteção", icon: ShieldCheck },
] as const;
document.title = "Lumina Ready";
void api.signalReady();
export default function App() {
  const [library, setLibrary] = useState<LibraryConfig | null | undefined>(),
    [view, setView] = useState<View>("dashboard"),
    [importOpen, setImportOpen] = useState(false),
    [jobId, setJobId] = useState<string>(),
    [jobs, setJobs] = useState<JobOverview[]>([]),
    [toast, setToast] = useState(""),
    previous = useRef(new Map<string, string>());
  useEffect(() => {
    api.getLibrary().then(setLibrary);
  }, []);
  useEffect(()=>{
    const error=(event:ErrorEvent)=>void api.recordClientError("frontend_error",event.message||"Erro não identificado");
    const rejection=(event:PromiseRejectionEvent)=>void api.recordClientError("unhandled_rejection",event.reason instanceof Error?event.reason.message:String(event.reason));
    window.addEventListener("error",error);
    window.addEventListener("unhandledrejection",rejection);
    return()=>{window.removeEventListener("error",error);window.removeEventListener("unhandledrejection",rejection)};
  },[]);
  useEffect(() => {
    if (!library) return;
    const load = () =>
      api
        .jobs()
        .then((next) => {
          next.forEach((j) => {
            const old = previous.current.get(j.jobId);
            if (
              old &&
              old !== j.state &&
              ["ready", "completed", "failed"].includes(j.state)
            )
              setToast(
                j.state === "ready"
                  ? `Análise de ${j.sourceName} pronta para revisão`
                  : j.state === "completed"
                    ? `Importação de ${j.sourceName} concluída`
                    : `Falha no trabalho de ${j.sourceName}`,
              );
            previous.current.set(j.jobId, j.state);
          });
          setJobs(next);
        })
        .catch(() => {});
    load();
    const timer = setInterval(load, 1000);
    return () => clearInterval(timer);
  }, [library]);
  if (library === undefined)
    return (
      <div className="splash">
        <LoaderCircle className="spin" />
        Preparando sua biblioteca…
      </div>
    );
  if (!library) return <Onboarding done={setLibrary} />;
  const openJob = (id: string) => {
      setJobId(id);
      setImportOpen(true);
    },
    active = jobs.filter((j) =>
      [
        "queued",
        "analyzing",
        "consolidating",
        "paused",
        "ready",
        "interrupted",
      ].includes(j.state),
    ).length;
  return (
    <div className="app-shell">
      <aside>
        <div className="brand">
          <div className="brand-mark">L</div>
          <div>
            <strong>Lumina</strong>
            <small>{library.name}</small>
          </div>
        </div>
        <button
          className="primary import-button"
          onClick={() => {
            setJobId(undefined);
            setImportOpen(true);
          }}
        >
          <Plus /> Nova importação
        </button>
        <nav>
          {nav.map((n) => (
            <button
              key={n.id}
              className={view === n.id ? "active" : ""}
              onClick={() => setView(n.id)}
            >
              <n.icon />
              <span>{n.label}</span>
              {n.id === "activity" && active > 0 && (
                <b className="nav-count">{active}</b>
              )}
            </button>
          ))}
        </nav>
        <div className="aside-footer">
          <Database />
          <div>
            <small>Acervo mestre</small>
            <span>{library.masterPath}</span>
          </div>
        </div>
      </aside>
      <main>
        <header className="topbar">
          <h1>{nav.find((n) => n.id === view)?.label}</h1>
          <div className="top-actions">
            <i className="status-dot" />
            Biblioteca saudável
          </div>
        </header>
        <JobBar jobs={jobs} open={openJob} />
        <section className="content">
          <Content
            view={view}
            navigate={setView}
            onImport={() => {
              setJobId(undefined);
              setImportOpen(true);
            }}
            jobs={jobs}
            openJob={openJob}
          />
        </section>
      </main>
      {importOpen && (
        <ImportWizard
          initialJobId={jobId}
          close={() => setImportOpen(false)}
          navigate={(v) => {
            setImportOpen(false);
            setView(v);
          }}
        />
      )}
      <Recovery />
      {toast && (
        <button className="app-toast" onClick={() => setToast("")}>
          {toast}
          <X />
        </button>
      )}
    </div>
  );
}
function Onboarding({ done }: { done: (x: LibraryConfig) => void }) {
  const [form, setForm] = useState({
      name: "Minha biblioteca",
      master: "D:\\Lumina\\Originais",
      backup: "G:\\Meu Drive\\Lumina Backup",
    }),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    try {
      done(await api.createLibrary(form.name, form.master, form.backup));
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  };
  return (
    <div className="onboarding">
      <div className="onboarding-copy">
        <div className="brand big">
          <div className="brand-mark">L</div>
          <strong>Lumina</strong>
        </div>
        <p className="eyebrow">SEU ACERVO, FINALMENTE SOB CONTROLE</p>
        <h1>Encontre, organize e proteja cada memória.</h1>
        <p>Consolide fotos e vídeos sem alterar as fontes.</p>
        <div className="trust-list">
          <span>
            <CheckCircle2 />
            Originais imutáveis
          </span>
          <span>
            <CheckCircle2 />
            Duplicação por conteúdo
          </span>
        </div>
      </div>
      <form className="setup-card" onSubmit={submit}>
        <p className="step">CONFIGURAÇÃO INICIAL</p>
        <h2>Crie sua biblioteca</h2>
        <label>
          Nome
          <input
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
        </label>
        <label>
          Pasta-mestre
          <input
            aria-label="Pasta-mestre"
            value={form.master}
            onChange={(e) => setForm({ ...form, master: e.target.value })}
          />
        </label>
        <label>
          Pasta de backup
          <input
            aria-label="Pasta de backup"
            value={form.backup}
            onChange={(e) => setForm({ ...form, backup: e.target.value })}
          />
        </label>
        {error && <p className="error">{error}</p>}
        <button className="primary" disabled={busy}>
          <Archive /> Criar biblioteca
        </button>
      </form>
    </div>
  );
}
const stageText = (stage: string, state?: string) =>
  state === "queued"
    ? "Aguardando para começar"
    : state === "paused"
      ? "Trabalho pausado"
      : state === "ready"
        ? "Análise pronta para revisar"
        : (
            {
              discovery: "Procurando fotos e vídeos",
              validation: "Verificando os arquivos",
              metadata: "Identificando datas e câmeras",
              hashing: "Comparando com sua biblioteca",
              deduplication: "Organizando os resultados",
              copying: "Copiando novos arquivos com segurança",
              thumbnail: "Preparando a galeria",
              backup: "Criando a cópia de proteção",
              backing_up: "Finalizando a cópia de proteção",
              sync_inventory: "Preparando atualização da fonte",
              sync_reconcile: "Comparando fonte e catálogo",
              completed: "Trabalho concluído",
            } as Record<string, string>
          )[stage] || "Processando suas mídias";
const stateText = (state: string) =>
  (
    ({
      queued: "Na fila",
      analyzing: "Em análise",
      consolidating: "Importando",
      pausing: "Pausando",
      paused: "Pausado",
      canceling: "Cancelando",
      canceled: "Cancelado",
      ready: "Pronto para revisar",
      interrupted: "Interrompido",
      failed: "Com erro",
      completed: "Concluído",
    }) as Record<string, string>
  )[state] || state;
const elapsedText = (start: string, end: string) => {
  const minutes = Math.floor(
    Math.max(0, new Date(end).getTime() - new Date(start).getTime()) / 60000,
  );
  return minutes < 1
    ? "menos de 1 min"
    : minutes < 60
      ? `${minutes} min`
      : `${Math.floor(minutes / 60)}h ${minutes % 60}min`;
};
function JobBar({
  jobs,
  open,
}: {
  jobs: JobOverview[];
  open: (x: string) => void;
}) {
  const j =
    jobs.find((x) =>
      ["queued", "analyzing", "consolidating", "paused"].includes(x.state),
    ) || jobs.find((x) => x.state === "ready");
  if (!j) return null;
  return (
    <button
      className={`global-job ${j.state === "ready" ? "ready" : ""}`}
      onClick={() => open(j.jobId)}
    >
      <div>
        <strong>{stageText(j.stage, j.state)}</strong>
        <span>
          {j.processedItems.toLocaleString("pt-BR")} de{" "}
          {j.totalItems.toLocaleString("pt-BR")} arquivos · {j.sourceName}
        </span>
      </div>
      <div className="mini-progress">
        <i style={{ width: `${j.overallPercent}%` }} />
      </div>
      <b>
        {j.state === "ready" ? "Revisar" : `${Math.round(j.overallPercent)}%`}
      </b>
    </button>
  );
}
function Content({
  view,
  navigate,
  onImport,
  jobs,
  openJob,
}: {
  view: View;
  navigate: (view: View) => void;
  onImport: () => void;
  jobs: JobOverview[];
  openJob: (x: string) => void;
}) {
  if (view === "library") return <Gallery />;
  if (view === "review") return <ReviewCenter navigate={navigate} />;
  if (view === "sources") return <Sources onImport={onImport} />;
  if (view === "duplicates") return <Duplicates />;
  if (view === "albums") return <Albums navigate={navigate} />;
  if (view === "activity")
    return <ActivityCenter jobs={jobs} openJob={openJob} />;
  if (view === "protection") return <Protection />;
  return <Dashboard onImport={onImport} navigate={navigate} />;
}
function ReviewCenter({ navigate }: { navigate: (view: View) => void }) {
  const [summary, setSummary] = useState<ReviewSummary>();
  const [notice,setNotice]=useState("");
  useEffect(() => { api.reviewSummary().then(setSummary); }, []);
  const open = (filters: Parameters<typeof openGalleryWithFilters>[0]) => {
    openGalleryWithFilters(filters);
    navigate("library");
  };
  const cards = [
    {label:"Revisar depois",value:summary?.reviewLater??0,detail:"Itens separados por você",action:()=>open({reviewLater:true})},
    {label:"Datas suspeitas",value:summary?.suspiciousDates??0,detail:"Datas obtidas do arquivo ou fora do intervalo esperado",action:()=>open({dateSuspicious:true})},
    {label:"Previews pendentes",value:summary?.missingPreviews??0,detail:"Miniaturas ausentes ou com falha",action:()=>navigate("protection")},
    {label:"Metadados incompletos",value:summary?.incompleteMetadata??0,detail:"Informações técnicas ainda não enriquecidas",action:()=>navigate("activity")},
    {label:"Falhas técnicas",value:summary?.technicalFailures??0,detail:"Previews ou metadados que precisam de uma nova tentativa",action:()=>navigate("activity")},
    {label:"Proteção pendente",value:summary?.pendingProtection??0,detail:"Mídias sem réplica verificada",action:()=>navigate("protection")},
    {label:"Duplicatas sem decisão",value:summary?.undecidedDuplicates??0,detail:"Grupos exatos aguardando revisão",action:()=>navigate("duplicates")},
  ];
  return <>
    <div className="section-heading"><div><h2>Central de revisão</h2><p>Tudo que merece uma decisão humana, reunido por prioridade.</p></div><div className="activity-actions"><button onClick={async()=>{const result=await api.rebuildCache();setNotice(`${result.generated} previews reparados · ${result.failed} falhas`);setSummary(await api.reviewSummary())}}>Reparar previews</button><button onClick={async()=>{await api.startFormatEnrichment();setNotice("Complementação de metadados iniciada em segundo plano. Acompanhe em Atividade.")}}>Completar metadados</button><button onClick={async()=>{const result=await api.undoLastEdit();setNotice(result.affected?"Última alteração desfeita.":"Nenhuma alteração para desfazer.");setSummary(await api.reviewSummary())}}>Desfazer última alteração</button><button onClick={()=>api.reviewSummary().then(setSummary)}><RefreshCw/>Atualizar</button></div></div>
    {notice&&<div className="notice" role="status">{notice}</div>}
    <div className="review-grid">{cards.map(card=><button key={card.label} onClick={card.action}><span>{card.label}</span><strong>{card.value.toLocaleString("pt-BR")}</strong><small>{card.detail}</small><ChevronRight/></button>)}</div>
  </>;
}
function ReportTools() {
  const [job, setJob] = useState(""),
    [notice, setNotice] = useState("");
  useEffect(() => {
    api.events().then((x) => setJob(x[0]?.jobId || ""));
  }, []);
  if (!job) return null;
  return (
    <div className="report-tools">
      <span>Relatórios técnicos</span>
      <button
        onClick={async () =>
          setNotice((await api.exportReport(job, "jsonl")).path)
        }
      >
        Exportar JSONL
      </button>
      <button
        onClick={async () =>
          setNotice((await api.exportReport(job, "csv")).path)
        }
      >
        Exportar CSV
      </button>
      <button
        onClick={async () =>
          setNotice(`${await api.retryFailed(job)} itens preparados`)
        }
      >
        Tentar novamente
      </button>
      {notice && (
        <p className="safe-note" role="status">
          {notice}
        </p>
      )}
    </div>
  );
}
const typeLabel = (x: string) =>
  (({ raw: "RAW", video: "Vídeos", photo: "Fotos" }) as Record<string, string>)[
    x
  ] || x;
const duration = (ms: number) =>
  ms < 60000
    ? `${Math.round(ms / 1000)}s`
    : `${Math.floor(ms / 60000)}min ${Math.round((ms % 60000) / 1000)}s`;
const normalizeEquipment = (value: string) => {
  const clean = value.trim().replace(/[_-]+/g, " ").replace(/\s+/g, " ");
  const aliases: [RegExp, string][] = [
    [/^dji\s*(fc\d+|mavic.*|mini.*|air.*|phantom.*)$/i, "DJI $1"],
    [/^iphone\s*/i, "Apple iPhone "],
    [/^sm\s+/i, "Samsung SM-"],
    [/^gopro\s*/i, "GoPro "],
  ];
  return aliases
    .reduce((name, [pattern, replacement]) => pattern.test(name) ? name.replace(pattern, replacement) : name, clean)
    .replace(/\b(dji|gopro)\b/gi, word => word.toUpperCase());
};
const monthRange = (key: string) => {
  const [year, month] = key.split("-").map(Number);
  return {
    dateFrom: `${key}-01`,
    dateTo: new Date(Date.UTC(year, month, 1)).toISOString().slice(0, 10),
  };
};
function Dashboard({ onImport,navigate }: { onImport: () => void;navigate:(view:View)=>void }) {
  const [s, setS] = useState<DashboardStats>(),[updating,setUpdating]=useState(false),[refreshError,setRefreshError]=useState(""),[inventoryMessage,setInventoryMessage]=useState("");
  const refresh=()=>{setUpdating(true);setRefreshError("");api.refreshDashboard().then(setS).catch(error=>setRefreshError(String(error))).finally(()=>setUpdating(false))};
  const openLibrary=(filters:Parameters<typeof openGalleryWithFilters>[0])=>{openGalleryWithFilters(filters);navigate("library")};
  useEffect(() => {
    let live=true;
    api.dashboard().then(snapshot=>{if(!live)return;setS(snapshot);setUpdating(true);return api.refreshDashboard()}).then(full=>{if(live&&full)setS(full)}).catch(error=>live&&setRefreshError(String(error))).finally(()=>live&&setUpdating(false));
    return()=>{live=false};
  }, []);
  if (!s) return <LoaderCircle className="spin" />;
  const coverage = (s.protected / Math.max(1, s.totalAssets)) * 100,
    storage=s.storage,
    recentMonths=[...s.months.slice(0,12)].reverse(),maxMonth=Math.max(1,...recentMonths.map(value=>value.bytes)),
    latestMonth=recentMonths.at(-1),previousMonth=recentMonths.at(-2),monthDelta=latestMonth&&previousMonth?((latestMonth.bytes-previousMonth.bytes)/Math.max(1,previousMonth.bytes))*100:undefined;
  const composition=[
    {key:"photo",label:"Fotos",items:s.photos,bytes:s.types.filter(x=>x.key!=="video").reduce((total,x)=>total+x.bytes,0)},
    {key:"video",label:"Vídeos",items:s.videos,bytes:s.types.find(x=>x.key==="video")?.bytes||0},
  ],compositionTotal=Math.max(1,composition.reduce((total,item)=>total+item.items,0)),photoPercent=composition[0].items/compositionTotal*100;
  const equipment=Object.values(s.cameras.reduce<Record<string,DashboardStats["cameras"][number]>>((all,item)=>{const key=normalizeEquipment(item.key);const current=all[key]||{key,items:0,bytes:0};current.items+=item.items;current.bytes+=item.bytes;all[key]=current;return all},{})).sort((a,b)=>b.bytes-a.bytes);
  const benchmarks=s.recentBenchmarks?.length?s.recentBenchmarks:(s.latestBenchmark?[s.latestBenchmark]:[]),maxBenchmark=Math.max(1,...benchmarks.map(item=>item.analysisMs+item.hashingMs+item.copyMs+item.thumbnailsMs));
  return (
    <>
      <div className="hero dashboard-hero">
        <div>
          <p className="eyebrow">VISÃO GERAL</p>
          <h2>
            {s.totalAssets.toLocaleString("pt-BR")} memórias ·{" "}
            {formatBytes(s.bytes)}
          </h2>
          <p>
            {s.oldest && s.newest
              ? `${formatDate(s.oldest)} até ${formatDate(s.newest)}`
              : "Sua biblioteca está pronta para crescer."}
          </p>
          <small className="dashboard-freshness">{updating?"Atualizando análises em segundo plano…":s.stale?"Exibindo snapshot salvo":"Atualizado agora"} · {new Date(s.snapshotGeneratedAt).toLocaleString("pt-BR")}</small>
          {refreshError&&<small className="dashboard-refresh-error">Uma seção não pôde ser atualizada. O snapshot continua disponível.</small>}
        </div>
        <div className="dashboard-actions">
          <button onClick={refresh} disabled={updating}><RefreshCw className={updating?"spin":""}/> Atualizar visão</button>
          <button className="primary" onClick={onImport}><Plus /> Importar mídia</button>
        </div>
      </div>
      {storage&&<section className="storage-intelligence">
        <div className="storage-heading"><div><p className="eyebrow">CAPACIDADE E PROTEÇÃO</p><h3>Onde suas memórias estão e quanto ainda cabe</h3></div><button onClick={()=>navigate("protection")}>Gerenciar proteção <ChevronRight/></button></div>
        <div className="memory-overview">
          <span className="memory-total"><small>Memórias preservadas</small><b>{s.totalAssets.toLocaleString("pt-BR")}</b><em>{s.photos.toLocaleString("pt-BR")} fotos · {s.videos.toLocaleString("pt-BR")} vídeos</em></span>
          <span><small>Primeira e última foto</small><b>{s.oldestPhoto?formatDate(s.oldestPhoto):"—"}</b><em>{s.newestPhoto?`até ${formatDate(s.newestPhoto)}`:"Sem fotos datadas"}</em></span>
          <span><small>Primeiro e último vídeo</small><b>{s.oldestVideo?formatDate(s.oldestVideo):"—"}</b><em>{s.newestVideo?`até ${formatDate(s.newestVideo)}`:"Sem vídeos datados"}</em></span>
          <span><small>Proteção verificada</small><b>{Math.round(coverage)}%</b><em>{s.pending.toLocaleString("pt-BR")} pendentes · {s.errors} erros</em></span>
        </div>
        <div className="storage-volumes">
          <StorageVolume title="Acervo principal" used={storage.masterUsedBytes} total={storage.masterTotalBytes} managed={storage.libraryBytes} detail={`${storage.estimatedAdditionalItems.toLocaleString("pt-BR")} mídias adicionais pela média atual`}/>
          <StorageVolume title="Réplica local" used={storage.backupUsedBytes} total={storage.backupTotalBytes} managed={Math.max(0,s.bytes-storage.pendingBackupBytes)} detail={storage.backupAvailable?`${formatBytes(Math.max(0,storage.projectedBackupFreeBytes))} livres após réplica e reserva`:"Destino indisponível"}/>
        </div>
        <div className="storage-facts"><span><small>Administrado pelo Lumina</small><b>{formatBytes(storage.libraryBytes)}</b></span><span><small>Pendente de proteção</small><b>{formatBytes(storage.pendingBackupBytes)}</b></span><span><small>Cache e temporários</small><b>{formatBytes(storage.cacheBytes+storage.temporaryBytes)}</b></span><span><small>Tamanho médio de arquivo</small><b>{formatBytes(storage.averageAssetBytes)}</b></span></div>
      </section>}
      <div className="dashboard-layout">
        <article className="panel composition">
          <p className="eyebrow">COMPOSIÇÃO</p>
          <h3>Fotos e vídeos</h3>
          <div className="composition-chart" style={{background:`conic-gradient(var(--green) 0 ${photoPercent}%,#d8a735 ${photoPercent}% 100%)`}}><span><b>{compositionTotal.toLocaleString("pt-BR")}</b><small>arquivos</small></span></div>
          <div className="composition-legend">{composition.map((x,index)=><button key={x.key} onClick={()=>openLibrary({mediaType:x.key})}><i className={index?"video":"photo"}/><span><b>{x.label}</b><small>{x.items.toLocaleString("pt-BR")} · {formatBytes(x.bytes)}</small></span><strong>{Math.round(x.items/compositionTotal*100)}%</strong></button>)}</div>
        </article>
        <article className="panel growth-panel">
          <p className="eyebrow">RITMO DO ACERVO</p>
          <h3>Volume capturado nos últimos 12 meses ativos</h3>
          <div className="growth-summary"><strong>{latestMonth?formatBytes(latestMonth.bytes):"—"}</strong><span>{latestMonth?.key||"Sem período"}{monthDelta!==undefined&&` · ${monthDelta>=0?"+":""}${Math.round(monthDelta)}% ante o mês ativo anterior`}</span></div>
          <div className="growth-bars">{recentMonths.map(value=><button key={value.key} title={`${value.key}: ${value.items.toLocaleString("pt-BR")} itens · ${formatBytes(value.bytes)}`} onClick={()=>openLibrary(monthRange(value.key))}><i style={{height:`${Math.max(5,value.bytes/maxMonth*100)}%`}}/><small>{value.key.slice(5)}</small></button>)}</div>
        </article>
        <article className="panel timeline-panel">
          <p className="eyebrow">LINHA DO TEMPO</p>
          <h3>Volume por ano</h3>
          <div className="year-bars">
            {s.years.slice(0, 8).map((x) => (
              <div key={x.key} role="button" tabIndex={0} onClick={()=>openLibrary({year:Number(x.key)})} onKeyDown={event=>event.key==="Enter"&&openLibrary({year:Number(x.key)})}>
                <b>{x.key}</b>
                <i>
                  <em
                    style={{
                      width: `${(x.bytes / Math.max(1, ...s.years.map((y) => y.bytes))) * 100}%`,
                    }}
                  />
                </i>
                <span>
                  {x.items.toLocaleString("pt-BR")} · {formatBytes(x.bytes)}
                </span>
              </div>
            ))}
          </div>
        </article>
        <article className="panel ranking-panel">
          <p className="eyebrow">INVENTÁRIO TÉCNICO</p>
          <h3>Inventário técnico</h3>
          <h4>Formatos de arquivo</h4>
          {s.formats.slice(0,6).map(x=><button key={x.key} onClick={()=>openLibrary({extension:x.key})}><span><b>{x.label}</b><small>{x.family} · suporte {x.support}</small></span><strong>{x.items.toLocaleString("pt-BR")} · {formatBytes(x.bytes)}</strong></button>)}
          <h4>Equipamentos normalizados</h4>
          {equipment.slice(0,5).map(x=><button className="dashboard-rank" key={x.key} onClick={()=>openLibrary({camera:x.key})}><span>{x.key}</span><b>{x.items.toLocaleString("pt-BR")}</b><small>{formatBytes(x.bytes)}</small></button>)}
          {!!s.codecs?.length&&<><h4>Codecs de vídeo</h4>{s.codecs.slice(0,4).map(x=><div className="dashboard-rank" key={x.key}><span>{x.key}</span><b>{x.items.toLocaleString("pt-BR")}</b><small>{formatBytes(x.bytes)}</small></div>)}</>}
          <button onClick={()=>api.startFormatEnrichment().then(id=>setInventoryMessage(`Inventário iniciado · ${id.slice(0,8)}`)).catch(error=>setInventoryMessage(String(error)))}>Atualizar inventário técnico</button>
          {inventoryMessage&&<small className="inventory-message">{inventoryMessage}</small>}
        </article>
        <article className="panel duplicate-overview">
          <p className="eyebrow">CONTEÚDO EM VÁRIAS ORIGENS</p>
          <h3>{s.duplicateGroups.toLocaleString("pt-BR")} grupos conhecidos</h3>
          <div><span><small>Ocorrências adicionais</small><b>{formatBytes(s.duplicateBytes)}</b></span><span><small>Com proteção suficiente para futura revisão</small><b>{formatBytes(s.reclaimableBytes)}</b></span></div>
          <p>Estimativa informativa baseada em conteúdo idêntico. Nenhum arquivo será removido automaticamente.</p>
          <button onClick={()=>navigate("duplicates")}>Analisar ocorrências <ChevronRight/></button>
        </article>
        <article className="panel insight-panel">
          <p className="eyebrow">PRECISA DA SUA ATENÇÃO</p>
          <h3>Insights da biblioteca</h3>
          {s.insights.length ? (
            s.insights.map((x) => (
              <button
                className={`dashboard-insight ${x.severity}`}
                key={`${x.kind}-${x.title}`}
                onClick={()=>x.kind==="dates"?openLibrary({dateSuspicious:true}):navigate(x.action)}
              >
                <span>
                  <b>{x.title}</b>
                  <small>{x.detail}</small>
                  <em>{x.reason} · confiança {x.confidence}</em>
                </span>
                <strong>
                  {x.bytes
                    ? formatBytes(x.bytes)
                    : x.value.toLocaleString("pt-BR")}
                </strong>
                <i>{x.actionLabel}<ChevronRight/></i>
              </button>
            ))
          ) : (
            <p className="all-good">Nenhuma pendência encontrada.</p>
          )}
        </article>
        {!!benchmarks.length && (
          <article className="panel benchmark-panel">
            <p className="eyebrow">DESEMPENHO MEDIDO</p>
            <h3>Comparativo dos últimos processamentos</h3>
            <div className="benchmark-chart">{benchmarks.map((job,index)=>{const stages=[job.analysisMs,job.hashingMs,job.copyMs,job.thumbnailsMs],total=stages.reduce((sum,value)=>sum+value,0);return <div className="benchmark-row" key={job.jobId}><span><b>{index===0?"Mais recente":`Anterior ${index}`}</b><small>{job.items.toLocaleString("pt-BR")} itens · {formatBytes(job.bytes)}</small></span><div className="benchmark-stack" style={{width:`${Math.max(8,total/maxBenchmark*100)}%`}}>{stages.map((value,stage)=><i key={stage} className={`stage-${stage}`} style={{width:`${total?value/total*100:0}%`}}/>)}</div><strong>{duration(total)}</strong></div>})}</div>
            <div className="benchmark-legend"><span><i className="stage-0"/>Análise</span><span><i className="stage-1"/>Leitura</span><span><i className="stage-2"/>Cópia</span><span><i className="stage-3"/>Previews</span></div>
          </article>
        )}
        <details className="panel dashboard-diagnostics">
          <summary>Diagnóstico da visão geral</summary>
          {s.timings.map(value=><span key={value.section}>{value.section}<b>{value.milliseconds} ms</b></span>)}
        </details>
      </div>
    </>
  );
}
function StorageVolume({title,used,total,managed,detail}:{title:string;used:number;total:number;managed:number;detail:string}){const percent=total?Math.min(100,used/total*100):0,managedPercent=total?Math.min(100,managed/total*100):0;return <article><header><span>{title}</span><strong>{formatBytes(Math.max(0,total-used))} livres</strong></header><div className="volume-bar"><i style={{width:`${percent}%`}}/><em style={{width:`${managedPercent}%`}}/></div><div><b>{formatBytes(used)} usados de {formatBytes(total)}</b><small>{detail}</small></div></article>}
function Stat({
  label,
  value,
  detail,
}: {
  label: string;
  value: string | number;
  detail?: string;
}) {
  return (
    <article>
      <span>{label}</span>
      <strong>
        {typeof value === "number" ? value.toLocaleString("pt-BR") : value}
      </strong>
      {detail && <small>{detail}</small>}
    </article>
  );
}
function Sources({ onImport }: { onImport: () => void }) {
  const [items, setItems] = useState<Source[]>([]);
  const [syncing, setSyncing] = useState<Record<string, JobProgress>>({});
  const [notice, setNotice] = useState("");
  const [syncingAll,setSyncingAll]=useState(false);
  useEffect(() => {
    api.sources().then(setItems);
  }, []);
  async function synchronize(source: Source) {
    setNotice("");
    try {
      const jobId = await api.startSourceSync(source.id);
      while (true) {
        const progress = await api.jobProgress(jobId);
        setSyncing((current) => ({ ...current, [source.id]: progress }));
        if (["completed", "failed", "canceled"].includes(progress.state)) {
          if (progress.state === "completed") {
            setNotice(`${source.name} atualizada: ${progress.imported} novas · ${progress.duplicates} duplicatas · ${progress.excluded} ausentes.`);
            setItems(await api.sources());
          } else {
            setNotice(`A sincronização de ${source.name} terminou como ${progress.state}.`);
          }
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
    } catch (cause) {
      setNotice(cause instanceof Error ? cause.message : String(cause));
    }
  }
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>De onde vêm suas mídias</h2>
          <p>Fontes continuam inventariadas quando offline.</p>
        </div>
        <div className="activity-actions"><button disabled={syncingAll||!items.some(source=>source.available)} onClick={async()=>{setSyncingAll(true);for(const source of items.filter(item=>item.available)){await synchronize(source)}setSyncingAll(false);setNotice("Todas as fontes conectadas foram atualizadas.")}}><RefreshCw className={syncingAll?"spin":""}/>{syncingAll?"Atualizando fontes":"Atualizar conectadas"}</button><button className="primary" onClick={onImport}><Plus />Adicionar fonte</button></div>
      </div>
      {notice && <div className="notice" role="status">{notice}</div>}
      <div className="source-list">
        {items.map((s) => {
          const progress = syncing[s.id];
          const active = progress && !["completed", "failed", "canceled"].includes(progress.state);
          return (
          <article key={s.id}>
            <HardDrive />
            <div className="source-main">
              <h3>{s.name}</h3>
              <p>
                {s.volumeLabel} · {s.path}
              </p>
            </div>
            <div>
              <small>Última análise</small>
              <strong>{formatDate(s.lastScan)}</strong>
            </div>
            <div>
              <small>Mídias encontradas</small>
              <strong>{s.assetCount}</strong>
            </div>
            <span className="badge">
              {s.available ? "Conectada" : "Offline"}
            </span>
            <button disabled={!s.available || active} onClick={() => synchronize(s)}>
              <RefreshCw className={active ? "spin" : ""} />
              {active ? `${Math.round(progress.overallPercent)}%` : "Atualizar"}
            </button>
          </article>
          );
        })}
      </div>
    </>
  );
}
function Duplicates() {
  const [items, setItems] = useState<DuplicateGroup[]>([]);
  const [status, setStatus] = useState<DuplicateStatus>();
  const [notice, setNotice] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [plan, setPlan] = useState<CleanupPlan>();

  async function loadDuplicates() {
    setLoading(true);
    setError("");
    try {
      const [groups, overview] = await Promise.all([api.duplicates(), api.duplicateStatus()]);
      setItems(groups);
      setStatus(overview);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }

  async function decide(
    group: DuplicateGroup,
    decision: "keep_all" | "review" | "remove_candidates",
    message: string,
  ) {
    await api.updateDuplicateDecision(group.assetId, decision, "user_review");
    setItems((current) =>
      current.map((item) =>
        item.assetId === group.assetId ? { ...item, decision } : item,
      ),
    );
    setNotice(message);
  }
  useEffect(() => {
    loadDuplicates();
  }, []);
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>Duplicatas exatas</h2>
          <p>Mesmo conteúdo agrupado; nada é excluído.</p>
        </div>
        <button onClick={loadDuplicates} disabled={loading}>
          <RefreshCw className={loading ? "spin" : ""} />
          Atualizar
        </button>
      </div>
      {notice && <p className="safe-note" role="status">{notice}</p>}
      {error && <div className="error-box">Não foi possível consultar duplicatas: {error}</div>}
      {status && <div className="duplicate-status" role="status">
        <span className={`status-pill ${status.state === "found" ? "warning" : "success"}`}>{status.state === "found" ? "Duplicatas encontradas" : status.state === "not_analyzed" ? "Aguardando catálogo" : "Análise concluída"}</span>
        <span><strong>{status.catalogAssets.toLocaleString("pt-BR")}</strong> mídias catalogadas</span>
        <span><strong>{status.occurrences.toLocaleString("pt-BR")}</strong> ocorrências ativas</span>
        <span><strong>{status.connectedSources}</strong> de {status.totalSources} fontes conectadas</span>
        <span>Última análise: <strong>{formatDate(status.lastScan)}</strong></span>
      </div>}
      <section className="cleanup-planner">
        <div><h3>Plano de limpeza seguro</h3><p>Simule candidatas e espaço potencial. Esta beta não remove arquivos.</p></div>
        <button className="primary" onClick={async()=>setPlan(await api.createCleanupPlan())}>Gerar plano</button>
        {plan&&<><div className="cleanup-summary"><span><strong>{plan.groups}</strong> grupos</span><span><strong>{plan.candidates}</strong> candidatas elegíveis</span><span><strong>{formatBytes(plan.bytes)}</strong> potencial</span><span><strong>{plan.blocked}</strong> bloqueadas</span></div><button onClick={async()=>{const report=await api.exportCleanupPlan(plan.id);setNotice(`Relatório exportado em ${report.path}`)}}>Exportar relatório do plano</button></>}
      </section>
      {!loading && !error && items.length === 0 && (
        <div className="empty-duplicates">
          <Copy />
          <h3>{status?.state === "not_analyzed" ? "A biblioteca ainda não foi analisada" : "Nenhuma duplicata exata encontrada"}</h3>
          <p>
            Um grupo aparece aqui depois que o mesmo conteúdo é encontrado em duas ou mais
            origens importadas. Fotos parecidas não são tratadas como duplicatas exatas.
          </p>
          <small>{status?.totalSources !== status?.connectedSources ? "Conecte e atualize as fontes offline para uma análise completa." : "A análise está atualizada. Fotos visualmente parecidas não são classificadas como cópias exatas."}</small>
        </div>
      )}
      <div className="duplicate-list">
        {items.map((g) => (
          <article key={g.hash}>
            <DuplicateThumb assetId={g.assetId} filename={g.filename} />
            <div className="duplicate-main">
              <h3>{g.filename}</h3>
              <p>{formatBytes(g.bytes)}</p>
              <p>{formatBytes(g.additionalBytes)} em ocorrências adicionais · {g.safety==="eligible_for_review"?`${formatBytes(g.reclaimableBytes)} aptos para futura revisão`:"proteja o acervo antes de decidir"}</p>
              <div className="occurrence-comparison" aria-label={`Comparação de ${g.filename}`}>
                {g.occurrences.map((o, i) => (
                  <section key={o.id}>
                    <DuplicateThumb assetId={g.assetId} filename={`${g.filename} em ${o.source}`} />
                    <strong><HardDrive />{i===0?"Referência":"Ocorrência adicional"}</strong>
                    <span>{o.source}<small>{o.path}</small></span>
                    <small>{g.safety==="eligible_for_review"?"Réplica verificada":"Proteção pendente"}</small>
                    <div><button className={o.decision==="keep"?"active":""} onClick={async()=>{await api.updateOccurrenceDecision(o.id,"keep");setItems(current=>current.map(group=>group.assetId===g.assetId?{...group,occurrences:group.occurrences.map(item=>item.id===o.id?{...item,decision:"keep"}:item)}:group))}}>Manter</button><button className={o.decision==="review"?"active":""} onClick={async()=>{await api.updateOccurrenceDecision(o.id,"review");setItems(current=>current.map(group=>group.assetId===g.assetId?{...group,occurrences:group.occurrences.map(item=>item.id===o.id?{...item,decision:"review"}:item)}:group))}}>Revisar</button><button disabled={i===0||g.safety!=="eligible_for_review"} className={o.decision==="remove_candidate"?"active":""} onClick={async()=>{await api.updateOccurrenceDecision(o.id,"remove_candidate");setItems(current=>current.map(group=>group.assetId===g.assetId?{...group,occurrences:group.occurrences.map(item=>item.id===o.id?{...item,decision:"remove_candidate"}:item)}:group))}}>Candidata</button></div>
                  </section>
                ))}
              </div>
              <div className="duplicate-actions">
                <button
                  className={g.decision === "keep_all" ? "active" : ""}
                  onClick={() =>
                    decide(g, "keep_all", "Grupo marcado para manter todas as cópias.")
                  }
                >
                  Manter todas
                </button>
                <button
                  className={g.decision === "review" ? "active" : ""}
                  onClick={() =>
                    decide(g, "review", "Grupo separado para revisão posterior.")
                  }
                >
                  Revisar depois
                </button>
                <button
                  className={g.decision === "remove_candidates" ? "active" : ""}
                  disabled={g.safety !== "eligible_for_review"}
                  title={
                    g.safety !== "eligible_for_review"
                      ? "Crie e verifique a réplica antes desta decisão."
                      : undefined
                  }
                  onClick={() =>
                    decide(
                      g,
                      "remove_candidates",
                      "Cópias adicionais marcadas como candidatas. Nenhum arquivo foi removido.",
                    )
                  }
                >
                  Marcar candidatas
                </button>
              </div>
            </div>
            <span className="count">{g.occurrences.length} cópias</span>
          </article>
        ))}
      </div>
    </>
  );
}
function DuplicateThumb({assetId,filename}:{assetId:string;filename:string}){const[src,setSrc]=useState<string|null>();useEffect(()=>{let live=true;api.thumbnail(assetId).then(value=>live&&setSrc(value));return()=>{live=false}},[assetId]);return <div className="duplicate-preview">{src?<img src={src} alt={`Prévia de ${filename}`}/>:<Copy/>}</div>}
function Albums({navigate}:{navigate:(view:View)=>void}) {
  const [items, setItems] = useState<Album[]>([]),
    [smart, setSmart] = useState<SavedView[]>([]),
    [tags, setTags] = useState<TagInfo[]>([]),
    [name, setName] = useState(""),
    [creating, setCreating] = useState(false);
  const load = () => api.albums().then(setItems);
  useEffect(() => {
    load();
    api.savedViews().then(views=>setSmart(views.filter(view=>view.smartAlbum)));
    api.tags().then(setTags);
  }, []);
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>Álbuns</h2>
          <p>Agrupe memórias sem duplicar originais.</p>
        </div>
        <button className="primary" onClick={() => setCreating(true)}>
          <Plus />
          Novo álbum
        </button>
      </div>
      {creating && (
        <div className="inline-create">
          <input
            aria-label="Nome do álbum"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <button onClick={() => setCreating(false)}>Cancelar</button>
          <button
            className="primary"
            onClick={async () => {
              await api.createAlbum(name);
              setCreating(false);
              setName("");
              load();
            }}
          >
            Criar álbum
          </button>
        </div>
      )}
      <div className="album-grid">
        {smart.map((view) => (
          <article key={view.id} className="smart-album" onClick={()=>{openGalleryWithFilters(view.filters);navigate("library")}}>
            <div><Search /></div>
            <h3>{view.name}</h3>
            <p>Álbum inteligente · atualizado automaticamente</p>
            <div className="album-actions"><button onClick={async event=>{event.stopPropagation();const name=prompt("Novo nome",view.name);if(name){await api.renameSavedView(view.id,name);setSmart(await api.savedViews().then(views=>views.filter(item=>item.smartAlbum)))}}}>Renomear</button><button onClick={async event=>{event.stopPropagation();if(confirm(`Excluir ${view.name}?`)){await api.deleteSavedView(view.id);setSmart(current=>current.filter(item=>item.id!==view.id))}}}>Excluir</button></div>
          </article>
        ))}
        {items.map((a) => (
          <article key={a.id} className="smart-album" onClick={()=>{openGalleryWithFilters({albumId:a.id});navigate("library")}}>
            <div>
              <AlbumIcon />
            </div>
            <h3>{a.name}</h3>
            <p>{a.assetCount} itens</p>
            <div className="album-actions"><button onClick={async event=>{event.stopPropagation();const name=prompt("Novo nome",a.name);if(name){await api.renameAlbum(a.id,name);load()}}}>Renomear</button><button onClick={async event=>{event.stopPropagation();if(confirm(`Excluir ${a.name}? As mídias permanecerão no acervo.`)){await api.deleteAlbum(a.id);load()}}}>Excluir</button></div>
          </article>
        ))}
      </div>
      <section className="tag-manager"><h3>Tags</h3><p>Renomeie ou remova classificações do catálogo sem alterar as mídias.</p><div>{tags.map(tag=><span key={tag.id}><button onClick={async()=>{const name=prompt("Novo nome da tag",tag.name);if(name){await api.renameTag(tag.id,name);setTags(await api.tags())}}}>{tag.name} · {tag.assetCount}</button><button aria-label={`Excluir tag ${tag.name}`} onClick={async()=>{if(confirm(`Remover a tag ${tag.name} do catálogo?`)){await api.deleteTag(tag.id);setTags(current=>current.filter(item=>item.id!==tag.id))}}}><X/></button></span>)}</div></section>
    </>
  );
}
function JobCenter({
  jobs,
  openJob,
}: {
  jobs: JobOverview[];
  openJob: (x: string) => void;
}) {
  const [events, setEvents] = useState<ImportEvent[]>([]),
    [notice, setNotice] = useState(""),
    [busy, setBusy] = useState("");
  useEffect(() => {
    const load = () => api.events().then(setEvents);
    load();
    const timer = setInterval(load, 1000);
    return () => clearInterval(timer);
  }, []);
  const act = async (
    j: JobOverview,
    action: "paused" | "running" | "canceled",
  ) => {
    if (
      action === "canceled" &&
      !confirm(
        "Cancelar este trabalho? O histórico e os arquivos já verificados serão preservados.",
      )
    )
      return;
    setBusy(j.jobId + action);
    try {
      await api.controlImport(j.jobId, action);
      setNotice(
        action === "paused"
          ? "Trabalho pausado"
          : action === "running"
            ? "Trabalho retomado"
            : "Cancelamento solicitado",
      );
    } catch (e) {
      setNotice(String(e));
    } finally {
      setBusy("");
    }
  };
  const sections = [
    {
      title: "Em andamento",
      items: jobs.filter((j) =>
        [
          "queued",
          "analyzing",
          "consolidating",
          "paused",
          "pausing",
          "canceling",
        ].includes(j.state),
      ),
    },
    {
      title: "Aguardando você",
      items: jobs.filter((j) =>
        ["ready", "interrupted", "failed"].includes(j.state),
      ),
    },
    {
      title: "Histórico",
      items: jobs.filter((j) => ["completed", "canceled"].includes(j.state)),
    },
  ];
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>Central de trabalhos</h2>
          <p>
            Acompanhe e controle análises e importações sem interromper sua
            navegação.
          </p>
        </div>
      </div>
      {notice && (
        <p className="safe-note" role="status">
          {notice}
        </p>
      )}
      {sections.map((section) => (
        <section className="job-section" key={section.title}>
          <h3>
            {section.title} <span>{section.items.length}</span>
          </h3>
          {!section.items.length ? (
            <p className="job-empty">Nenhum trabalho nesta seção.</p>
          ) : (
            <div className="job-list">
              {section.items.map((j) => (
                <article key={j.jobId}>
                  <div>
                    <strong>{stageText(j.stage, j.state)}</strong>
                    <small>
                      {j.sourceName} · {j.sourcePath}
                    </small>
                  </div>
                  <span className={`badge ${j.state}`}>
                    {j.state === "ready"
                      ? "Revisar"
                      : j.state === "interrupted"
                        ? "Interrompido"
                        : j.state === "completed"
                          ? "Concluído"
                          : j.state}
                  </span>
                  <div className="mini-progress">
                    <i style={{ width: `${j.overallPercent}%` }} />
                  </div>
                  <b>
                    {j.processedItems.toLocaleString("pt-BR")}/
                    {j.totalItems.toLocaleString("pt-BR")}
                  </b>
                  <div className="job-controls">
                    {["analyzing", "consolidating"].includes(j.state) && (
                      <button
                        disabled={!!busy}
                        onClick={() => act(j, "paused")}
                      >
                        Pausar
                      </button>
                    )}
                    {j.state === "paused" && (
                      <button
                        disabled={!!busy}
                        onClick={() => act(j, "running")}
                      >
                        Retomar
                      </button>
                    )}
                    {j.state === "interrupted" && (
                      <button
                        disabled={!!busy}
                        onClick={async () => {
                          setBusy(j.jobId);
                          await api.resumeJob(j.jobId);
                          setBusy("");
                        }}
                      >
                        Retomar
                      </button>
                    )}
                    {[
                      "queued",
                      "analyzing",
                      "consolidating",
                      "paused",
                    ].includes(j.state) && (
                      <button
                        className="danger-button"
                        disabled={!!busy}
                        onClick={() => act(j, "canceled")}
                      >
                        Cancelar
                      </button>
                    )}
                    <button onClick={() => openJob(j.jobId)}>
                      {j.state === "ready" ? "Revisar" : "Detalhes"}
                    </button>
                    {j.failed > 0 && (
                      <button
                        onClick={async () =>
                          setNotice(
                            `${await api.retryFailed(j.jobId)} itens preparados`,
                          )
                        }
                      >
                        Repetir falhas
                      </button>
                    )}
                  </div>
                </article>
              ))}
            </div>
          )}
        </section>
      ))}
      <details className="event-history">
        <summary>Diagnósticos e eventos ({events.length})</summary>
        <div className="timeline">
          {events.map((e) => (
            <article key={e.id}>
              <CheckCircle2 />
              <div>
                <strong>{e.details}</strong>
                <small>{e.path}</small>
              </div>
              <time>{formatDate(e.at)}</time>
            </article>
          ))}
        </div>
      </details>
    </>
  );
}
function Protection() {
  const [result, setResult] = useState(""),
    [config, setConfig] = useState<LibraryConfig>(),
    [backup, setBackup] = useState(""),
    [master, setMaster] = useState(""),
    [thumbnailHealth, setThumbnailHealth] = useState<ThumbnailAudit>(),
    [repairing, setRepairing] = useState(false),
    [repairProgress,setRepairProgress]=useState<ThumbnailRepairProgress>(),
    [health,setHealth]=useState<LibraryHealth>(),
    [queue, setQueue] = useState<{
      pending: number;
      failed: number;
      pendingBytes: number;
    }>();
  useEffect(() => {
    api.getLibrary().then((x) => {
      if (x) {
        setConfig(x);
        setBackup(x.backupPath);
        setMaster(x.masterPath);
      }
    });
    api.protectionQueue().then(setQueue);
    api.auditThumbnails(false).then(setThumbnailHealth).catch(() => {});
    api.libraryHealth().then(setHealth).catch(()=>{});
  }, []);
  useEffect(()=>{
    if(!repairing)return;
    const poll=()=>api.thumbnailRepairProgress().then(setRepairProgress).catch(()=>{});
    poll();
    const timer=window.setInterval(poll,350);
    return()=>window.clearInterval(timer);
  },[repairing]);
  const choose = async (set: (x: string) => void) => {
    const p = await api.chooseFolder();
    if (p) set(p);
  };
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>Proteção e armazenamento</h2>
          <p>Gerencie a cópia-mestre e a fila independente de proteção.</p>
        </div>
        <div className="activity-actions">
          <button
            onClick={async () => {
              const report = await api.exportDiagnostics();
              setResult("Diagnóstico seguro exportado em " + report.path);
            }}
          >
            Exportar diagnóstico
          </button>
          <button
            onClick={async () =>
              setResult(
                `${await api.clearCache()} miniaturas removidas do cache`,
              )
            }
          >
            Limpar cache
          </button>
          <button
            onClick={async () => {
              const r = await api.rebuildCache();
              setResult(
                `${r.generated} miniaturas geradas · ${r.failed} falhas`,
              );
            }}
          >
            Reconstruir miniaturas
          </button>
          <button
            className="primary"
            onClick={async () => {
              await api.verifyBackup();
              setResult("Verificação iniciada em segundo plano. Acompanhe, pause ou cancele em Atividade.");
            }}
          >
            <ShieldCheck />
            Verificar agora
          </button>
        </div>
      </div>
      {result && (
        <div className="notice" role="status">
          {result}
        </div>
      )}
      {health&&<section className={`health-overview ${health.overall}`}>
        <header>
          <div><span className={`status-pill ${health.overall === "healthy" ? "success" : health.overall === "attention" ? "warning" : "error"}`}>{health.overall === "healthy" ? "Tudo saudável" : health.overall === "attention" ? "Requer atenção" : "Ação necessária"}</span><h3>Saúde da biblioteca</h3><p>{health.checks.filter(check=>check.state!=="healthy").length ? `${health.checks.filter(check=>check.state!=="healthy").length} pontos merecem sua atenção.` : "Catálogo, mídias e ferramentas operando normalmente."}</p></div>
          <div className="health-score"><strong>{health.checks.filter(check=>check.state==="healthy").length}/{health.checks.length}</strong><span>verificações saudáveis</span></div>
        </header>
        <div className="health-actions">{health.checks.filter(check=>check.state!=="healthy"&&!['exiftool','ffmpeg','ffprobe'].includes(check.key)).map(check=><article key={check.key} className={check.state}><span className={`status-dot ${check.state}`}/><div><strong>{check.label}</strong><small>{check.detail}</small></div><span className={`status-pill ${check.state}`}>{check.state==="warning"?"Atenção":"Erro"}</span></article>)}</div>
        <details className="health-technical"><summary>Ver detalhes técnicos e verificações saudáveis</summary><div>{health.checks.filter(check=>check.state==="healthy"||['exiftool','ffmpeg','ffprobe'].includes(check.key)).map(check=><p key={check.key}><span>{check.label}</span><strong>{check.detail}</strong><i className={`status-dot ${check.state}`}/></p>)}</div></details>
      </section>}
      <div className="protection-flow">
        <article>
          <Database />
          <h3>Acervo mestre</h3>
          <p>{config?.masterPath}</p>
        </article>
        <ChevronRight />
        <article>
          <Folder />
          <h3>Réplica local</h3>
          <p>{config?.backupPath}</p>
        </article>
        <ChevronRight />
        <article>
          <Cloud />
          <h3>Nuvem Google</h3>
          <p>Estado desconhecido</p>
        </article>
      </div>
      <div className="storage-settings">
        <article>
          <h3>Saúde das miniaturas</h3>
          <p>
            <strong>{thumbnailHealth?.valid ?? 0}</strong> válidas ·{" "}
            <strong>{(thumbnailHealth?.missing ?? 0) + (thumbnailHealth?.stale ?? 0) + (thumbnailHealth?.corrupt ?? 0)}</strong> para reparar
          </p>
          {repairing && (
            <div className="thumbnail-repair-progress" role="progressbar" aria-label="Reparo de miniaturas em andamento">
              <span>Verificando {repairProgress?.processed || 0} de {repairProgress?.total || thumbnailHealth?.total || 0} mídias</span>
              <div><i style={{width:`${repairProgress?.total ? Math.round(repairProgress.processed/repairProgress.total*100) : 2}%`}} /></div>
              <small>{repairProgress?.regenerated || 0} recuperadas · {repairProgress?.failed || 0} falhas · a galeria continua disponível</small>
            </div>
          )}
          <button
            disabled={repairing || !thumbnailHealth || thumbnailHealth.valid === thumbnailHealth.total}
            onClick={async () => {
              setRepairing(true);
              setResult("Reparo de miniaturas em andamento. A galeria continua disponível.");
              try {
                const audit = await api.auditThumbnails(true);
                const finalHealth = await api.auditThumbnails(false);
                setThumbnailHealth(finalHealth);
                setResult(
                  `Reparo concluído: ${finalHealth.valid} de ${finalHealth.total} miniaturas válidas · ${audit.regenerated} recuperadas · ${audit.failed} falhas.`,
                );
              } catch (cause) {
                setResult(`Falha no reparo: ${cause instanceof Error ? cause.message : String(cause)}`);
              } finally {
                setRepairing(false);
              }
            }}
          >
            <RefreshCw className={repairing ? "spin" : ""} />
            {repairing ? "Reparando…" : "Reparar pendências"}
          </button>
        </article>
        <article>
          <h3>Fila de proteção</h3>
          <p>
            <strong>{queue?.pending ?? 0}</strong> pendentes ·{" "}
            {formatBytes(queue?.pendingBytes ?? 0)} ·{" "}
            <strong>{queue?.failed ?? 0}</strong> falhas
          </p>
        </article>
        <article>
          <h3>Alterar réplica</h3>
          <div className="folder-input">
            <input
              aria-label="Novo destino da réplica"
              value={backup}
              onChange={(e) => setBackup(e.target.value)}
            />
            <button onClick={() => choose(setBackup)}>
              <FolderOpen />
              Escolher
            </button>
          </div>
          <button
            onClick={async () => {
              const next = await api.updateBackupPath(backup);
              setConfig(next);
              setResult(
                "Destino da réplica atualizado; itens pendentes foram preservados.",
              );
            }}
          >
            Salvar réplica
          </button>
        </article>
        <article>
          <h3>Migrar acervo-mestre</h3>
          <p>
            Todos os originais serão copiados e verificados antes da troca do
            catálogo.
          </p>
          <div className="folder-input">
            <input
              aria-label="Novo destino do acervo"
              value={master}
              onChange={(e) => setMaster(e.target.value)}
            />
            <button onClick={() => choose(setMaster)}>
              <FolderOpen />
              Escolher
            </button>
          </div>
          <button
            disabled={master === config?.masterPath}
            onClick={async () => {
              if (
                !confirm(
                  "Migrar e verificar integralmente o acervo? O local atual será preservado.",
                )
              )
                return;
              const m = await api.migrateMaster(master);
              setResult(
                `Migração concluída: ${m.processedItems} arquivos verificados. O acervo anterior foi preservado.`,
              );
              setConfig((await api.getLibrary()) || undefined);
            }}
          >
            Migrar com verificação
          </button>
        </article>
      </div>
    </>
  );
}
function Recovery() {
  const [jobs, setJobs] = useState<RecoverableJob[]>([]);
  useEffect(() => {
    api
      .recoverableJobs()
      .then(setJobs)
      .catch(() => {});
  }, []);
  const j = jobs[0];
  if (!j) return null;
  return (
    <div className="modal-backdrop">
      <div className="modal">
        <h2>Uma importação foi interrompida</h2>
        <p>{j.sourcePath}</p>
        <p>{j.interruptionReason}</p>
        <div className="modal-actions">
          <button
            onClick={async () => {
              await api.discardJob(j.jobId);
              setJobs((v) => v.slice(1));
            }}
          >
            Descartar trabalho
          </button>
          <button
            className="primary"
            onClick={async () => {
              await api.resumeJob(j.jobId);
              setJobs((v) => v.slice(1));
            }}
          >
            Retomar com segurança
          </button>
        </div>
      </div>
    </div>
  );
}
function ImportWizard({
  close,
  navigate,
  initialJobId,
}: {
  close: () => void;
  navigate: (x: View) => void;
  initialJobId?: string;
}) {
  const [step, setStep] = useState<
      "source" | "analysis" | "summary" | "progress" | "protection" | "done"
    >("source"),
    [path, setPath] = useState(
      () => localStorage.getItem("lumina:last-source") || "E:\\DCIM",
    ),
    [name, setName] = useState("Nova fonte"),
    [summary, setSummary] = useState<ImportSummary>(),
    [plan, setPlan] = useState<StoragePlan>(),
    [progress, setProgress] = useState<JobProgress>(),
    [error, setError] = useState(""),
    [activeJob, setActiveJob] = useState<string>();
  useEffect(() => {
    if (!activeJob || !("__TAURI_INTERNALS__" in window)) return;
    let stop: (() => void) | undefined;
    listen<JobProgress>(
      "job-progress",
      (e) => e.payload.jobId === activeJob && setProgress(e.payload),
    ).then((x) => (stop = x));
    return () => stop?.();
  }, [activeJob]);
  const loadReview = async (id: string) => {
    setSummary(await api.importSummary(id));
    setPlan(await api.storagePlan(id));
    setStep("summary");
  };
  const hydrate = async (id: string) => {
    const p = await api.jobProgress(id);
    setProgress(p);
    if (["ready", "waiting_space", "batch_pending"].includes(p.state)) {
      await loadReview(id);
      if (p.state === "waiting_space")
        setError(
          "Não há espaço suficiente. A análise foi preservada; libere espaço e verifique novamente.",
        );
      if (p.state === "batch_pending")
        setError(
          "O lote anterior está no acervo. Selecione o próximo lote pendente.",
        );
    } else if (
      p.state === "protection_pending" ||
      p.state === "waiting_backup_space"
    )
      setStep("protection");
    else if (p.state === "completed") setStep("done");
    else if (
      [
        "consolidating",
        "protecting",
        "paused",
        "pausing",
        "canceling",
      ].includes(p.state)
    ) {
      setSummary(await api.importSummary(id));
      setStep("progress");
    } else if (["queued", "analyzing"].includes(p.state)) setStep("analysis");
  };
  useEffect(() => {
    if (!initialJobId) return;
    setActiveJob(initialJobId);
    hydrate(initialJobId);
    const timer = setInterval(() => hydrate(initialJobId), 1000);
    return () => clearInterval(timer);
  }, [initialJobId]);
  const wait = async (id: string, states: string[]) => {
    for (;;) {
      const p = await api.jobProgress(id);
      setProgress(p);
      if (states.includes(p.state)) return p;
      await new Promise((r) => setTimeout(r, 250));
    }
  };
  const choose = async () => {
    const selected = await api.chooseFolder();
    if (!selected) return;
    setPath(selected);
    localStorage.setItem("lumina:last-source", selected);
    const suggested = selected.split(/[\\/]/).filter(Boolean).pop();
    if (suggested) setName(suggested);
  };
  const analyze = async () => {
    if (!path.trim()) {
      setError("Escolha uma pasta para analisar.");
      return;
    }
    setStep("analysis");
    setError("");
    localStorage.setItem("lumina:last-source", path);
    try {
      const id = await api.startAnalysis(path, name.trim() || "Nova fonte");
      setActiveJob(id);
      const p = await wait(id, ["ready", "failed", "canceled"]);
      if (p.state !== "ready") throw Error(p.state);
      await loadReview(id);
    } catch (e) {
      setError(String(e));
      setStep("source");
    }
  };
  const recheck = async () => {
    if (!summary) return;
    const next = await api.storagePlan(summary.jobId);
    setPlan(next);
    setError(
      next.canConsolidate
        ? "Espaço confirmado. A consolidação pode começar."
        : `Ainda faltam ${formatBytes(next.missingBytes)}.`,
    );
  };
  const selectBatch = async (mode: string, value?: string) => {
    if (!summary) return;
    await api.selectImport(
      summary.jobId,
      mode,
      value,
      mode === "maximum_safe" ? plan?.maximumSafeBytes : undefined,
    );
    setPlan(await api.storagePlan(summary.jobId));
    setError("");
  };
  const consolidate = async () => {
    if (!summary) return;
    const next = await api.storagePlan(summary.jobId);
    setPlan(next);
    if (!next.canConsolidate) {
      setError(
        `Faltam ${formatBytes(next.missingBytes)}. A análise está preservada e não será refeita.`,
      );
      return;
    }
    setStep("progress");
    try {
      await api.startConsolidation(summary.jobId);
      const p = await wait(summary.jobId, [
        "completed",
        "protection_pending",
        "batch_pending",
        "failed",
        "canceled",
        "waiting_space",
      ]);
      if (p.state === "completed") setStep("done");
      else if (p.state === "protection_pending") setStep("protection");
      else if (p.state === "batch_pending") {
        await loadReview(summary.jobId);
        setError(
          "Lote consolidado. Selecione os itens restantes para continuar.",
        );
      } else if (p.state === "waiting_space") {
        await loadReview(summary.jobId);
        setError("O espaço disponível mudou. A análise foi preservada.");
      } else throw Error(p.state === "canceled" ? "JOB_CANCELED" : p.state);
    } catch (e) {
      setError(
        String(e).includes("JOB_CANCELED")
          ? "Importação cancelada. Arquivos já verificados permanecem seguros e o histórico foi preservado."
          : String(e),
      );
      setStep("summary");
    }
  };
  const protect = async () => {
    if (!summary) return;
    setStep("progress");
    await api.startProtection(summary.jobId);
    const p = await wait(summary.jobId, [
      "completed",
      "backup_error",
      "waiting_backup_space",
      "canceled",
    ]);
    if (p.state === "completed") setStep("done");
    else {
      setProgress(p);
      setStep("protection");
    }
  };
  const control = async (action: "paused" | "running" | "canceled") =>
      summary && setProgress(await api.controlImport(summary.jobId, action)),
    percent = Math.round(progress?.overallPercent || 0),
    stagePercent = Math.round(progress?.stagePercent || 0);
  return (
    <div className="modal-backdrop">
      <div className="modal">
        <button
          aria-label="Fechar importação"
          className="icon-only close"
          onClick={close}
        >
          <X />
        </button>
        {step === "source" && (
          <>
            <p className="eyebrow">NOVA IMPORTAÇÃO · 1 DE 2</p>
            <h2>Escolha uma fonte</h2>
            <p>A primeira etapa apenas analisa. Nada será alterado.</p>
            <label>
              Nome da fonte
              <input value={name} onChange={(e) => setName(e.target.value)} />
            </label>
            <label>
              Pasta ou unidade
              <div className="folder-input">
                <input
                  placeholder="Escolha uma pasta no computador"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                />
                <button type="button" onClick={choose}>
                  <FolderOpen />
                  Escolher pasta…
                </button>
              </div>
            </label>
            {error && <p className="error">{error}</p>}
            <div className="modal-actions">
              <button onClick={close}>Cancelar</button>
              <button
                className="primary"
                disabled={!path.trim()}
                onClick={analyze}
              >
                <Search />
                Analisar fonte
              </button>
            </div>
          </>
        )}
        {step === "analysis" && (
          <div className="analyzing">
            <LoaderCircle className="spin" />
            <h2>
              {progress?.state === "queued"
                ? "Análise aguardando na fila"
                : "Analisando em segundo plano…"}
            </h2>
            <p className="analysis-stage">
              {progress
                ? stageText(progress.stage, progress.state)
                : "Preparando descoberta…"}
            </p>
            {progress && (
              <>
                <div
                  className="progress import-bar"
                  role="progressbar"
                  aria-label="Progresso da etapa de análise"
                  aria-valuenow={stagePercent}
                >
                  <i style={{ width: `${stagePercent}%` }} />
                </div>
                <p>
                  {progress.stage === "metadata"
                    ? progress.currentFile ||
                      "Preparando informações das mídias"
                    : `${progress.processedItems.toLocaleString("pt-BR")} de ${progress.totalItems.toLocaleString("pt-BR")} arquivos`}
                </p>
              </>
            )}
            <p>
              Você pode continuar navegando. O trabalho permanecerá visível em
              Atividade.
            </p>
            <button onClick={close}>Continuar navegando</button>
          </div>
        )}
        {step === "summary" && summary && (
          <>
            <p className="eyebrow">NOVA IMPORTAÇÃO · 2 DE 2</p>
            <h2>
              {plan && !plan.canConsolidate
                ? "Escolha o que cabe com segurança"
                : "Pronto para consolidar"}
            </h2>
            <div className="summary-grid">
              <span>
                <strong>{plan?.selectedItems ?? summary.newFiles}</strong>{" "}
                selecionados
              </span>
              <span>
                <strong>{summary.duplicates}</strong> duplicatas exatas
              </span>
              <span>
                <strong>{summary.invalid}</strong> para revisar
              </span>
              <span>
                <strong>
                  {formatBytes(plan?.selectedBytes ?? summary.requiredBytes)}
                </strong>{" "}
                no acervo
              </span>
            </div>
            {summary.issues.length > 0 && (
              <div className="import-issues" aria-label="Itens que precisam de revisão">
                <strong>O que precisa de atenção</strong>
                {summary.issues.map((issue) => (
                  <div className="import-issue" key={`${issue.kind}-${issue.extension}`}>
                    <span>{issue.items} arquivos .{issue.extension.toUpperCase()} · {formatBytes(issue.bytes)}</span>
                    <small>{issue.message} Eles não serão copiados; a origem permanece intacta.</small>
                  </div>
                ))}
              </div>
            )}
            <div className="batch-options">
              <button onClick={() => selectBatch("all")}>Tudo</button>
              <button onClick={() => selectBatch("media_type", "photo")}>
                Fotos
              </button>
              <button onClick={() => selectBatch("media_type", "raw")}>
                RAW
              </button>
              <button onClick={() => selectBatch("media_type", "video")}>
                Vídeos
              </button>
              {plan && (
                <button onClick={() => selectBatch("maximum_safe")}>
                  Máximo seguro ({plan.maximumSafeItems})
                </button>
              )}
            </div>
            {plan && (
              <div className={plan.canConsolidate ? "safe-note" : "error"}>
                <HardDrive />
                <span>
                  Acervo: {formatBytes(plan.masterAvailableBytes)} livres ·
                  Réplica: {formatBytes(plan.backupAvailableBytes)} livres
                  {plan.sameVolume ? " · mesma unidade física" : ""}
                  {!plan.canConsolidate && (
                    <>
                      {" "}
                      · faltam <strong>{formatBytes(plan.missingBytes)}</strong>
                    </>
                  )}
                </span>
              </div>
            )}
            <div className="safe-note">
              <ShieldCheck />A análise está salva e suas fontes não serão
              alteradas.
            </div>
            {error && (
              <p className={plan?.canConsolidate ? "safe-note" : "error"}>
                {error}
              </p>
            )}
            <div className="modal-actions">
              <button onClick={() => setStep("source")}>Voltar</button>
              {plan && !plan.canConsolidate && (
                <button onClick={recheck}>Verificar espaço novamente</button>
              )}
              <button
                className="primary"
                disabled={plan ? !plan.canConsolidate : true}
                onClick={consolidate}
              >
                <Archive />
                Consolidar {plan?.selectedItems ?? summary.newFiles} itens
              </button>
            </div>
          </>
        )}
        {step === "progress" && (
          <div className="import-progress">
            <p className="eyebrow">IMPORTAÇÃO EM ANDAMENTO</p>
            <h2>
              {progress?.state === "paused"
                ? "Importação pausada"
                : "Consolidando com segurança"}
            </h2>
            <div
              className="progress import-bar"
              role="progressbar"
              aria-label="Progresso geral da importação"
              aria-valuenow={percent}
            >
              <i style={{ width: `${percent}%` }} />
            </div>
            <div
              className="progress import-bar"
              role="progressbar"
              aria-label="Progresso da etapa"
              aria-valuenow={stagePercent}
            >
              <i style={{ width: `${stagePercent}%` }} />
            </div>
            <p>{progress?.currentFile}</p>
            <div className="state-grid">
              <span>
                Acervo: <strong>{progress?.libraryState}</strong>
              </span>
              <span>
                Backup: <strong>{progress?.backupState}</strong>
              </span>
            </div>
            <div className="modal-actions">
              <button
                onClick={() =>
                  control(progress?.state === "paused" ? "running" : "paused")
                }
              >
                {progress?.state === "paused" ? "Retomar" : "Pausar"}
              </button>
              <button onClick={() => control("canceled")}>
                Cancelar importação
              </button>
            </div>
          </div>
        )}
        {step === "protection" && (
          <div className="done">
            <CheckCircle2 />
            <p className="eyebrow">ACERVO CONSOLIDADO</p>
            <h2>As mídias já estão disponíveis na galeria.</h2>
            <p>
              A cópia de proteção é independente. Você pode iniciá-la agora ou
              continuar navegando.
            </p>
            {progress?.state === "waiting_backup_space" && (
              <p className="error">
                A réplica está sem espaço. O acervo-mestre permanece seguro e a
                fila foi preservada.
              </p>
            )}
            <div className="modal-actions">
              <button onClick={() => navigate("library")}>Ver galeria</button>
              <button className="primary" onClick={protect}>
                <ShieldCheck />
                Proteger agora
              </button>
            </div>
          </div>
        )}
        {step === "done" && (
          <div className="done">
            <CheckCircle2 />
            <p className="eyebrow">IMPORTAÇÃO CONCLUÍDA</p>
            <h2>Suas mídias foram verificadas.</h2>
            <div className="modal-actions">
              <button onClick={() => navigate("activity")}>
                Ver atividade
              </button>
              <button className="primary" onClick={() => navigate("library")}>
                Ver mídias importadas
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
