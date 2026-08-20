# Rastreabilidade 0.5

| Requisito | Implementação | Validação |
|---|---|---|
| Trabalho visível ao navegar | `list_jobs`, `GlobalJobBar`, `JobCenter` | testes React + build |
| Retomar/reabrir | hidratação por `get_job_progress` | fluxo React |
| Fila de análises | `JobManager.pending_analysis` | teste Rust de cancelamento da fila |
| Grade/lista | `Gallery` + `localStorage` | teste React de alternância/persistência |
| Paginação/virtualização/cache | cursor SQLite, TanStack Virtual, LRU 300 | testes e benchmark 100k legado |
| Tags/álbuns/data em lote | comandos transacionais Tauri | compilação Rust + teste React de tag |
| Datas suspeitas | filtro SQL e sinalização visual | teste Rust do predicado |
| Miniaturas | cache, preview e auditoria/reparo | teste React e suíte Rust de mídia |
