import type { JobOverview } from "./types";

export type JobBucket = "active" | "attention" | "history";

export const jobStateLabel: Record<string,string> = {
  queued:"Na fila",analyzing:"Em análise",consolidating:"Importando",protecting:"Protegendo",
  pausing:"Pausando…",paused:"Pausado",canceling:"Cancelando…",ready:"Pronto para revisar",
  waiting_space:"Sem espaço no acervo",batch_pending:"Próximo lote disponível",
  protection_pending:"Importação concluída · proteção pendente",waiting_backup_space:"Sem espaço na réplica",
  backup_error:"Falha na proteção",interrupted:"Interrompido",failed:"Com erro",completed:"Concluído",canceled:"Cancelado",
};

const activeStates=new Set(["queued","analyzing","consolidating","protecting","pausing","canceling"]);
const attentionStates=new Set(["paused","ready","waiting_space","batch_pending","protection_pending","waiting_backup_space","backup_error","interrupted","failed"]);

export function jobBucket(state:string):JobBucket {
  if(activeStates.has(state))return "active";
  if(attentionStates.has(state))return "attention";
  return "history";
}

export function jobNextStep(job:JobOverview):string {
  return ({
    ready:"A análise terminou. Revise a seleção para iniciar a importação.",
    paused:"Este trabalho está parado por sua solicitação. Retome ou cancele quando quiser.",
    protection_pending:"Os arquivos já estão no acervo. Inicie a réplica quando o destino estiver disponível.",
    batch_pending:"O lote atual terminou. Você pode preparar o próximo lote.",
    waiting_space:"Libere espaço ou escolha outra localização para continuar.",
    waiting_backup_space:"A importação está segura no acervo, mas a réplica precisa de espaço.",
    backup_error:"A importação terminou; repita somente a etapa de proteção.",
    interrupted:"O aplicativo foi fechado durante este trabalho. A retomada é segura.",
    failed:"Consulte o motivo e tente novamente apenas os itens com falha.",
    completed:"Importação e etapas solicitadas encerradas.",
    canceled:"Trabalho encerrado por solicitação do usuário.",
  } as Record<string,string>)[job.state] || "O trabalho está avançando automaticamente.";
}

export const isJobPollingFast=(jobs:JobOverview[])=>jobs.some(job=>jobBucket(job.state)==="active");

export type JobHeartbeat = "current" | "waiting" | "stalled";

export function jobHeartbeat(job:JobOverview,now=Date.now()):JobHeartbeat {
  if(jobBucket(job.state)!=="active")return "current";
  const updated=new Date(job.updatedAt).getTime();
  if(!Number.isFinite(updated))return "current";
  const age=Math.max(0,now-updated);
  if(job.state==="queued")return age>=120_000?"waiting":"current";
  // A file copy can legitimately take minutes. A durable processing item is
  // stronger evidence of liveness than the UI timestamp alone.
  if((job.queueProcessing??0)>0)return "current";
  return age>=180_000?"stalled":"current";
}

export function heartbeatLabel(job:JobOverview,now=Date.now()):string {
  const seconds=Math.floor(Math.max(0,now-new Date(job.updatedAt).getTime())/1000);
  const duration=seconds<60?`${seconds}s`:`${Math.floor(seconds/60)} min`;
  return jobHeartbeat(job,now)==="waiting"?`Na fila há ${duration}`:`Fila sem worker ativo há ${duration}`;
}
