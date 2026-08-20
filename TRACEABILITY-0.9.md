# Rastreabilidade 0.9

| Compromisso | Implementação | Evidência |
|---|---|---|
| 1 Deduplicação por tamanho | `size_counts`, índice `assets(bytes,hash)`, candidatos seletivos | `unique_sizes_defer_hash_until_verified_copy` e pipeline completo |
| 2 Armazenamento adaptativo | `storage_profile`, unidade removível e perfil conservador | `adaptive_hashing_uses_a_bounded_pool_and_returns_every_hash` |
| 3 Hash durante cópia | `copy_hash_to_temp_verified`, `promote_verified_temp` | teste de cópia integrada e teste de tamanhos únicos |
| 4 Metadados unificados | ExifTool com metadados e validação no mesmo lote de até 200 | testes de lote, cancelamento e RAW |
| 5 Miniaturas progressivas | `work_queue(kind='thumbnail')`, processamento posterior e geração sob demanda | pipeline completo e auditoria de miniaturas |
| 6 Telemetria | métricas de perfil, hashing seletivo, cache, análise, cópia e miniatura | dashboard de benchmark e testes de progresso |
| 7 Progresso confiável | conclusão atômica por worker, bytes, velocidade, ETA e estágio ativo | testes de modelos, jobs e retomada |
| 8 Dashboard rico | composição, cronologia, proteção, espaço, fontes e benchmark | testes React e contrato `DashboardStats` |
| 9 Insights | proteção, ocorrências, miniaturas, datas e fontes offline | consulta consolidada do dashboard |
| 10 Agregados e índices | `library_rollups`, triggers e índices compostos/parciais | teste com 100 mil mídias e conferência de rollup |
