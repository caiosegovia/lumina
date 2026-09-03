import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, ChevronDown, Clock3, FileDown, Pause, Play, RotateCcw, XCircle } from "lucide-react";
import { api } from "./api";
import { formatBytes, formatDate } from "./format";
import type { ImportEvent, JobOverview } from "./types";
import { jobBucket, jobNextStep, jobStateLabel } from "./jobState";
import "./activity.css";

const stageLabel: Record<string, string> = {
  discovery: "Inventariando pastas", inventory:"Inventário rápido", confirmation:"Confirmando conteúdo", metadata: "Identificando datas e câmeras",
  validation: "Verificando os arquivos", hashing: "Comparando com sua biblioteca",
  technical_enrichment: "Organizando formatos e detalhes técnicos",
  deduplication: "Organizando os resultados", copying: "Copiando com segurança",
  thumbnail: "Preparando a galeria", backup: "Criando a cópia de proteção",
  verification: "Conferindo a integridade da réplica", verification_error: "Revisando falhas da réplica",
  backing_up: "Finalizando a cópia de proteção", ready: "Análise pronta",
  space_check: "Espaço insuficiente para continuar",
  protection_pending:"Acervo pronto; proteção pendente", backup_space_check:"Aguardando espaço na réplica",
  completed: "Importação concluída",
  sync_inventory:"Preparando atualização da fonte",sync_reconcile:"Comparando fonte e catálogo"
};

function elapsed(job: JobOverview) {
  const minutes = Math.floor(Math.max(0, new Date(job.updatedAt).getTime() - new Date(job.createdAt).getTime()) / 60000);
  if (minutes < 1) return "menos de 1 min";
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)}h ${minutes % 60}min`;
}
function eta(seconds?: number) {
  if (seconds == null || seconds < 0) return "estimando tempo";
  if (seconds < 60) return "menos de 1 min restante";
  const minutes = Math.ceil(seconds / 60);
  return minutes < 60 ? `cerca de ${minutes} min restantes` : `cerca de ${Math.floor(minutes / 60)}h ${minutes % 60}min restantes`;
}

export default function ActivityCenter({ jobs, openJob }: { jobs: JobOverview[]; openJob: (id: string) => void }) {
  const [events, setEvents] = useState<ImportEvent[]>([]);
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState("");
  const [historyOpen, setHistoryOpen] = useState(false);
  const [storageWarning, setStorageWarning] = useState("");
  const jobRevision=jobs.map(job=>`${job.jobId}:${job.state}:${job.updatedAt}`).join("|");
  useEffect(() => { api.events().then(setEvents); }, [jobRevision]);
  useEffect(() => { api.getLibrary().then(library => { if (!library) return; const master=library.masterPath.match(/^[A-Za-z]:/)?.[0].toUpperCase(),backup=library.backupPath.match(/^[A-Za-z]:/)?.[0].toUpperCase(); if(master&&master===backup)setStorageWarning(`O acervo e a réplica estão na mesma unidade ${master}. Isso reduz a velocidade e não protege contra falha física do disco.`); }); }, []);
  useEffect(() => { if (!notice) return; const id = setTimeout(() => setNotice(""), 4500); return () => clearTimeout(id); }, [notice]);

  const act = async (job: JobOverview, action: "paused" | "running" | "canceled") => {
    if (action === "canceled" && !confirm("Cancelar este trabalho? As mídias de origem não serão alteradas.")) return;
    setBusy(job.jobId + action);
    try {
      await api.controlImport(job.jobId, action);
      setNotice(action === "paused" ? "Pausa solicitada" : action === "running" ? "Trabalho retomado" : "Cancelamento solicitado; aguardando a etapa atual encerrar com segurança");
    } catch (error) { setNotice(String(error)); } finally { setBusy(""); }
  };
  const active = jobs.filter(j => jobBucket(j.state)==="active");
  const attention = jobs.filter(j => jobBucket(j.state)==="attention");
  const history = jobs.filter(j => jobBucket(j.state)==="history");
  const latestJob = jobs[0]?.jobId || events[0]?.jobId || "";

  const card = (job: JobOverview, compact = false) => (
    <article className={`work-card state-${job.state} ${compact ? "compact" : ""}`} key={job.jobId}>
      <div className="work-icon">{job.state === "completed" ? <CheckCircle2/> : job.state === "failed" ? <AlertTriangle/> : job.state === "canceled" ? <XCircle/> : <Clock3/>}</div>
      <div className="work-main">
        <div className="work-title"><strong>{stageLabel[job.stage] || "Processando suas mídias"}</strong><span className={`work-state ${job.state}`}>{jobStateLabel[job.state] || job.state}</span></div>
        <p>{job.sourceName}<span>•</span>{job.sourcePath}</p>
        {jobBucket(job.state)==="active"&&<div className="work-progress" role="progressbar" aria-label={`Progresso de ${job.sourceName}`} aria-valuenow={Math.round(job.overallPercent)}><i style={{ width: `${job.overallPercent}%` }}/></div>}
        <div className="work-meta"><span>{formatBytes(job.processedBytes)} de {formatBytes(job.totalBytes)}</span><span>{Math.round(job.overallPercent)}%</span><span>{job.bytesPerSecond?`${formatBytes(job.bytesPerSecond)}/s · ${eta(job.estimatedSecondsRemaining)}`:`${job.processedItems.toLocaleString("pt-BR")} de ${job.totalItems.toLocaleString("pt-BR")} arquivos`}</span><span>{job.state==="completed"?elapsed(job):formatDate(job.updatedAt)}</span></div>
        <p className="work-next">{jobNextStep(job)}</p>
        {job.interruptionReason && <p className="work-reason">{job.interruptionReason}</p>}
      </div>
      <div className="work-actions">
        {["analyzing", "consolidating", "protecting"].includes(job.state) && <button disabled={!!busy} onClick={() => act(job, "paused")}><Pause/>Pausar</button>}
        {job.state === "paused" && <button disabled={!!busy} onClick={() => act(job, "running")}><Play/>Retomar</button>}
        {job.state === "interrupted" && <button disabled={!!busy} onClick={async () => { setBusy(job.jobId); try { await api.resumeJob(job.jobId); setNotice("Trabalho retomado"); } finally { setBusy(""); } }}><Play/>Retomar</button>}
        {["queued", "analyzing", "consolidating", "paused"].includes(job.state) && <button className="subtle-danger" disabled={!!busy} onClick={() => act(job, "canceled")}>Cancelar</button>}
        {job.state === "canceled" && <button onClick={async () => { await api.resumeJob(job.jobId); setNotice("Nova tentativa iniciada"); }}><RotateCcw/>Tentar novamente</button>}
        <button className="secondary" onClick={() => openJob(job.jobId)}>{job.state === "waiting_space" ? "Resolver espaço" : job.state === "ready" ? "Revisar" : "Detalhes"}</button>
      </div>
    </article>
  );

  return <div className="activity-center">
    <div className="activity-hero"><div><p className="eyebrow">TRABALHOS E IMPORTAÇÕES</p><h2>Atividade da biblioteca</h2><p>Acompanhe o que está acontecendo e o que precisa da sua atenção.</p></div><div className="activity-summary"><span><strong>{active.length}</strong> em andamento</span><span><strong>{attention.length}</strong> aguardando você</span></div></div>
    {storageWarning&&<div className="storage-warning"><AlertTriangle/><div><strong>Desempenho e proteção reduzidos</strong><p>{storageWarning}</p></div></div>}
    {notice && <button className="activity-notice" role="status" onClick={() => setNotice("")}>{notice}<XCircle/></button>}
    {active.length > 0 && <section className="work-section"><h3>Em andamento <b>{active.length}</b></h3><div className="work-cards">{active.map(j => card(j))}</div></section>}
    {attention.length > 0 && <section className="work-section"><h3>Precisa da sua atenção <b>{attention.length}</b></h3><div className="work-cards">{attention.map(j => card(j))}</div></section>}
    {active.length === 0 && attention.length === 0 && <div className="activity-empty"><CheckCircle2/><div><strong>Tudo em ordem</strong><p>Nenhum trabalho precisa da sua atenção.</p></div></div>}
    {history.length > 0 && <section className="history-section"><button className="history-toggle" onClick={() => setHistoryOpen(v => !v)} aria-expanded={historyOpen}><div><h3>Histórico recente</h3><span>{history.length} trabalhos encerrados</span></div><ChevronDown/></button>{historyOpen && <div className="work-cards history-cards">{history.map(j => card(j, true))}</div>}</section>}
    <details className="technical-tools"><summary>Diagnósticos e relatórios</summary><p>Informações para suporte e conferência da biblioteca.</p><div><button disabled={!latestJob} onClick={async () => setNotice((await api.exportReport(latestJob, "jsonl")).path)}><FileDown/>Exportar relatório completo</button><button disabled={!latestJob} onClick={async () => setNotice((await api.exportReport(latestJob, "csv")).path)}><FileDown/>Exportar planilha CSV</button></div>{events.length > 0 && <details><summary>{events.length} eventos registrados</summary><div className="diagnostic-events">{events.slice(0, 50).map(e => <p key={e.id}><time>{formatDate(e.at)}</time><span>{e.details}</span></p>)}</div></details>}</details>
  </div>;
}
