# Rastreabilidade 0.8

| Bloco | Implementação principal | Evidência de teste |
|---|---|---|
| 1 Planejador | `engine::storage_plan`, `StoragePlan`, assistente | testes de lotes e falta de espaço existentes |
| 2 Parcial | `job_selection`, `apply_selection`, `batch_pending` | `partial_batches_preserve_the_remaining_analysis` |
| 3 Backup independente | `work_queue`, `protect_job`, `protection_stats` | pipeline completo e teste de lotes |
| 4 Destinos/migração | `update_backup_path`, `library::migrate_master` | `migration_copies_and_verifies_before_switching_catalog_paths` |
| 5 Performance | `hash_files_adaptive`, buffer 8 MiB, cache incremental | `adaptive_hashing_uses_a_bounded_pool_and_returns_every_hash` e carga de 2 mil arquivos |
| 6 Duas etapas | estágio `inventory`, evento `inventory_ready`, confirmação profunda | métricas do pipeline e testes de análise |
| 7 Jobs | estados persistentes, controles e retomada da fila | testes de cancelamento, retomada e interrupção |
| 8 Galeria | catálogo antes da proteção, badges e filtros de proteção | testes da galeria e auditoria de miniaturas |
| 9 Consistência | contadores pós-cópia, eventos de seleção, TIFF não suportado | testes de modelos, validação e pipeline |

O relatório final de execução está em `TEST-REPORT-0.8.md`.
