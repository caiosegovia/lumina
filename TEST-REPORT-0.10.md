# Relatório de aceite — Lumina 0.10.0

Escopo: os 22 bloqueios arquiteturais levantados para uma biblioteca local de dezenas ou centenas de milhares de mídias. `OK` exige implementação e evidência automatizada ou benchmark reproduzível; compilação isolada não basta.

| # | Resultado | Implementação | Evidência |
|---|---|---|---|
| 1 | OK | Leitura de miniatura não gera mídia; galeria e painel executam consultas em worker assíncrono | `thumbnail_read_never_generates_work_inside_the_ui_request`; teste front “continua navegável…” |
| 2 | OK | Migração/schema executados uma vez por caminho de catálogo | `schema_initialization_runs_once_per_catalog` |
| 3 | OK | Gate global de I/O com prioridade interativa e limite de processos externos | `global_io_limit_is_enforced`; `interactive_io_precedes_background_waiters`; `enforces_global_process_concurrency_limit` |
| 4 | OK | Descoberta/validação/hash em `job_items`; miniatura/backup/verificação em `work_queue`; visão unificada `durable_work` | `durable_work_exposes_every_pipeline_family`; `thumbnail_read_never_generates_work_inside_the_ui_request` |
| 5 | OK | Fila paralela de análise em memória removida; reinício devolve `processing` para `pending` e oferece retomada | `restart_recovers_processing_work_from_the_catalog`; `interrupted_analysis_resumes_without_duplicate_items` |
| 6 | OK | WAL, busy timeout, inicialização separada e escrita em lotes | `wal_readers_remain_available_during_a_write_transaction`; `schema_initialization_runs_once_per_catalog` |
| 7 | OK | Fontes e tags de toda página são carregadas em duas consultas agrupadas | `page_relations_use_two_batched_queries_instead_of_n_plus_one` |
| 8 | OK | Intervalos de data/ano e extensão normalizada preservam índices compostos | `timeline_filter_uses_composite_index`; `filters_and_stats` |
| 9 | OK | Miniaturas servidas por protocolo local com cache HTTP; Base64 removido do transporte produtivo | `thumbnail_protocol_rejects_paths_and_accepts_catalog_ids`; teste front de URL `lumina-thumb` |
| 10 | OK | Orientação EXIF comum é lida nativamente; RAW usa fallback; processos externos têm limite global | `raw_validation_is_reused_from_the_metadata_batch`; `enforces_global_process_concurrency_limit`; regressão real `IMG_0268.CR2` |
| 11 | OK | Uma única enumeração `WalkDir`; processamento posterior usa o inventário em memória/persistido | `analysis_enumerates_the_physical_source_once`; métrica `inventory_walks=1` |
| 12 | OK | Fonte usa GUID/serial do volume e caminho relativo, mantendo mount path apenas para acesso | `equal_mount_paths_on_distinct_volumes_have_distinct_keys`; `windows_volume_identity_is_stable` |
| 13 | OK | Snapshot usa API de backup SQLite durante WAL ativo | `sqlite_backup_contains_committed_wal_data` |
| 14 | OK | Falhas de réplica/manifesto são propagadas e impedem `completed`/`replica_verified` | `backup_failure_never_reports_the_asset_as_protected`; triggers de invariantes |
| 15 | OK | Manifesto versionado, ordenado, com checksum do payload e promoção atômica | `manifest_is_versioned_complete_and_checksummed`; `atomic_metadata_write_replaces_the_complete_previous_version` |
| 16 | OK | Verificação é job persistente, controlável, limitada por I/O e atualizada em lotes | `verification_job_resumes_from_its_persistent_queue_after_cancel` |
| 17 | OK | Eventos de auditoria permanecem; telemetria concluída é compactada para janela de 50 mil | `telemetry_retention_never_deletes_failures` |
| 18 | OK | Transições de controle são transacionais; triggers impedem conclusão sem artefato verificado | `durable_queue_rejects_completion_without_its_invariant`; `rejects_invalid_state_jump` |
| 19 | OK | `JobState` e `WorkState` centralizam parsing e serialização dos estados persistidos | `every_persisted_state_round_trips` |
| 20 | OK | Progresso de hash/cópia/backup/verificação persiste a cada 8 itens ou no final; polling reduzido | `progress_persistence_is_batched_for_large_jobs`: 188 escritas para 1.500 itens |
| 21 | OK | Catálogo sintético com 100 mil registros, paginação real, p50/p95 e working set | `catalog_handles_one_hundred_thousand_assets`: p50 1 ms, p95 1 ms, working set 12.550.144 bytes nesta máquina |
| 22 | OK | Front testa interação enquanto miniatura está pendente; backend mede consulta durante geração concorrente | teste front “continua navegável…”; `gallery_queries_remain_fast_while_thumbnails_are_generated`; smoke portátil isolado |

## Gates finais

- Rust: 76 testes catalogados; suíte executada em uma thread para evitar interferência entre benchmarks.
- Frontend: 12/12.
- RAW real: 1/1 com `D:\Galeria Caio\2022\11\IMG_0268.CR2`.
- Build TypeScript/Vite: OK.
- `npm audit --audit-level=moderate`: 0 vulnerabilidades.
- MSI e NSIS: gerados pelo bundler oficial do Tauri.
- Portável: deve passar `scripts/smoke-portable-0.10.ps1`; o resultado final é registrado antes da entrega.
