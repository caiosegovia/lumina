# Rastreabilidade do Lumina 0.2

“Validado” exige implementação, teste executado e evidência em `TEST-REPORT-0.2.md`.

| ID | Implementação/evidência | Estado |
| --- | --- | --- |
| JOB-01 | `jobs.rs`; navegação durante análise | Validado |
| JOB-02 | SQLite `jobs`, `job_items`, `job_counters` | Validado |
| JOB-03 | reserva única e `LibraryLock` | Validado |
| JOB-04 | job único e `PROCESS_LIMIT=2` | Validado |
| JOB-05 | estágios persistidos e pipeline idempotente | Validado |
| JOB-06 | pausa cooperativa e retomada UI | Validado |
| JOB-07 | temporário `.lumina/temp/<job>` e teste de descarte | Validado |
| JOB-08 | interrupção, Retomar/Descartar e teste UI | Validado |
| JOB-09 | fontes byte a byte intactas | Validado |
| PRG-01 | snapshot completo `JobProgress` | Validado |
| PRG-02 | `library_state`/`backup_state` separados | Validado |
| PRG-03 | métricas calculadas/persistidas no Rust | Validado |
| PRG-04 | evento `job-progress` mais snapshot SQLite | Validado |
| PROC-01 | único `Command::new` em `process.rs` | Validado |
| PROC-02 | `CREATE_NO_WINDOW`, pipes e PE Windows GUI | Validado |
| PROC-03 | timeout, cancelamento e limite testados | Validado |
| PROC-04 | `process_events` com diagnóstico sanitizado | Validado |
| PROC-05 | sanitização e erro acionável testados | Validado |
| VAL-01 | decoder real e extensão falsa | Validado |
| VAL-02 | FFprobe/FFmpeg com MP4 e HEIC reais | Validado |
| VAL-03 | ExifTool/prévia com DNG real | Validado |
| VAL-04 | enum completo de estados | Validado |
| VAL-05 | revisão persistida e fonte intacta | Validado |
| THM-01 | JPEG, orientação, frame e prévia | Validado |
| THM-02 | chave hash/versão e reúso | Validado |
| THM-03 | leitura restrita ao cache catalogado | Validado |
| THM-04 | limpeza/reconstrução backend e UI | Validado |
| ACT-01 | atividade e filtros UI | Validado |
| ACT-02 | recibo JSONL/CSV paginado | Validado |
| ACT-03 | retry automático e diagnóstico copiável | Validado |
| ACT-04 | fonte offline versus arquivo ausente | Validado |
| ARC-01 | módulos separados do núcleo | Validado |
| ARC-02 | schema v4 e conexões curtas | Validado |
| ARC-03 | destino/temporário persistidos uma vez | Validado |
| ARC-04 | lock exclusivo testado | Validado |
| TST-01 | testes unitários Rust | Validado |
| TST-02 | integração em diretórios temporários | Validado |
| TST-03 | 6 testes frontend | Validado |
| TST-04 | 100 mil, 2 mil e 64 MiB | Validado |
| REL-01 | PE GUI, fonte intacta, promoção verificada | Validado |
| REL-02 | release, MSI, NSIS e portátil com hashes | Validado |
| REL-03 | matriz, relatório, aceite e limites | Validado |
