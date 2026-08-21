import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Activity,
  Album as AlbumIcon,
  Archive,
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
  DashboardStats,
  DuplicateGroup,
  ImportEvent,
  ImportSummary,
  JobOverview,
  JobProgress,
  LibraryConfig,
  RecoverableJob,
  Source,
  StoragePlan,
  View,
} from "./types";
const nav = [
  { id: "dashboard", label: "Visão geral", icon: LayoutDashboard },
  { id: "library", label: "Biblioteca", icon: Images },
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
  if (view === "sources") return <Sources onImport={onImport} />;
  if (view === "duplicates") return <Duplicates />;
  if (view === "albums") return <Albums />;
  if (view === "activity")
    return <ActivityCenter jobs={jobs} openJob={openJob} />;
  if (view === "protection") return <Protection />;
  return <Dashboard onImport={onImport} navigate={navigate} />;
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
    maxType = Math.max(1, ...s.types.map((x) => x.bytes));
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
      <div className="dashboard-big">
        <Stat
          label="Biblioteca"
          value={s.totalAssets}
          detail={`${s.types.map((x) => `${x.items} ${typeLabel(x.key)}`).join(" · ")}`}
        />
        <Stat
          label="Armazenamento usado"
          value={formatBytes(s.bytes)}
          detail={`${formatBytes(s.masterAvailableBytes)} livres no acervo`}
        />
        <Stat
          label="Proteção verificada"
          value={`${Math.round(coverage)}%`}
          detail={`${s.pending} pendentes · ${s.errors} erros`}
        />
        <Stat
          label="Período"
          value={
            s.years.length
              ? `${s.years.length} ${s.years.length === 1 ? "ano" : "anos"}`
              : "—"
          }
          detail={
            s.oldest && s.newest
              ? `${new Date(s.oldest).getFullYear()}–${new Date(s.newest).getFullYear()}`
              : "Sem datas"
          }
        />
      </div>
      <div className="dashboard-layout">
        <article className="panel composition">
          <p className="eyebrow">COMPOSIÇÃO</p>
          <h3>O que ocupa sua biblioteca</h3>
          {s.types.map((x) => (
            <div className="composition-row" key={x.key}>
              <span>
                <b>{typeLabel(x.key)}</b>
                <small>{x.items.toLocaleString("pt-BR")} itens</small>
              </span>
              <i>
                <em style={{ width: `${(x.bytes / maxType) * 100}%` }} />
              </i>
              <strong>{formatBytes(x.bytes)}</strong>
            </div>
          ))}
        </article>
        <article className="panel protection-card">
          <p className="eyebrow">PROTEÇÃO E ESPAÇO</p>
          <h3>
            {s.protected.toLocaleString("pt-BR")} de{" "}
            {s.totalAssets.toLocaleString("pt-BR")} protegidas
          </h3>
          <div className="progress">
            <i style={{ width: `${coverage}%` }} />
          </div>
          <dl>
            <div>
              <dt>Acervo livre</dt>
              <dd>{formatBytes(s.masterAvailableBytes)}</dd>
            </div>
            <div>
              <dt>Backup livre</dt>
              <dd>{formatBytes(s.backupAvailableBytes)}</dd>
            </div>
          </dl>
          {!!s.protectionYears?.length&&<small>Mais protegido por período: {s.protectionYears.slice(0,3).map(value=>`${value.key} (${value.items.toLocaleString("pt-BR")})`).join(" · ")}</small>}
          {!!s.protectionSources?.length&&<small>Origens protegidas: {s.protectionSources.slice(0,2).map(value=>`${value.key} (${value.items.toLocaleString("pt-BR")})`).join(" · ")}</small>}
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
          <p className="eyebrow">FORMATOS E EQUIPAMENTOS</p>
          <h3>Inventário técnico</h3>
          {s.formats.slice(0,6).map(x=><button key={x.key} onClick={()=>openLibrary({extension:x.key})}><span><b>{x.label}</b><small>{x.family} · suporte {x.support}</small></span><strong>{x.items.toLocaleString("pt-BR")} · {formatBytes(x.bytes)}</strong></button>)}
          <h4>Principais câmeras</h4>
          {s.cameras.slice(0,4).map(x=><div className="dashboard-rank" key={x.key}><span>{x.key}</span><b>{x.items.toLocaleString("pt-BR")}</b><small>{formatBytes(x.bytes)}</small></div>)}
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
        {s.latestBenchmark && (
          <article className="panel benchmark-panel">
            <p className="eyebrow">ÚLTIMO PROCESSAMENTO</p>
            <h3>Desempenho medido</h3>
            <div>
              <span>
                Análise<strong>{duration(s.latestBenchmark.analysisMs)}</strong>
              </span>
              <span>
                Leitura profunda
                <strong>{duration(s.latestBenchmark.hashingMs)}</strong>
              </span>
              <span>
                Cópia verificada
                <strong>{duration(s.latestBenchmark.copyMs)}</strong>
              </span>
              <span>
                Hash adiado
                <strong>
                  {s.latestBenchmark.deferredHashItems.toLocaleString("pt-BR")}
                </strong>
              </span>
            </div>
            <p>
              {formatBytes(s.latestBenchmark.hashedBytes)} exigiram leitura
              antecipada · {s.latestBenchmark.cacheHits} resultados reutilizados
            </p>
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
  useEffect(() => {
    api.sources().then(setItems);
  }, []);
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>De onde vêm suas mídias</h2>
          <p>Fontes continuam inventariadas quando offline.</p>
        </div>
        <button className="primary" onClick={onImport}>
          <Plus />
          Adicionar fonte
        </button>
      </div>
      <div className="source-list">
        {items.map((s) => (
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
          </article>
        ))}
      </div>
    </>
  );
}
function Duplicates() {
  const [items, setItems] = useState<DuplicateGroup[]>([]);
  useEffect(() => {
    api.duplicates().then(setItems);
  }, []);
  return (
    <>
      <div className="section-heading">
        <div>
          <h2>Duplicatas exatas</h2>
          <p>Mesmo conteúdo agrupado; nada é excluído.</p>
        </div>
      </div>
      <div className="duplicate-list">
        {items.map((g) => (
          <article key={g.hash}>
            <div className="duplicate-preview">
              <Copy />
            </div>
            <div className="duplicate-main">
              <h3>{g.filename}</h3>
              <p>{formatBytes(g.bytes)}</p>
              <p>{formatBytes(g.additionalBytes)} em ocorrências adicionais · {g.safety==="eligible_for_review"?`${formatBytes(g.reclaimableBytes)} aptos para futura revisão`:"proteja o acervo antes de decidir"}</p>
              <div>
                {g.occurrences.map((o, i) => (
                  <span key={i}>
                    <HardDrive />
                    {o.source}
                    <small>{o.path}</small>
                  </span>
                ))}
              </div>
            </div>
            <span className="count">{g.occurrences.length} cópias</span>
          </article>
        ))}
      </div>
    </>
  );
}
function Albums() {
  const [items, setItems] = useState<Album[]>([]),
    [name, setName] = useState(""),
    [creating, setCreating] = useState(false);
  const load = () => api.albums().then(setItems);
  useEffect(() => {
    load();
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
        {items.map((a) => (
          <article key={a.id}>
            <div>
              <AlbumIcon />
            </div>
            <h3>{a.name}</h3>
            <p>{a.assetCount} itens</p>
          </article>
        ))}
      </div>
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
  }, []);
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
