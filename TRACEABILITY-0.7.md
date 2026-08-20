# Rastreabilidade 0.7

| Objetivo | Implementação | Evidência |
|---|---|---|
| Remover custo RAW repetido | validação ExifTool no lote de metadados | teste `raw_validation_is_reused_from_the_metadata_batch` |
| Reanálise rápida | reutilização por caminho+tamanho+modificação+SHA-256 | teste de reimportação idempotente |
| Progresso confiável | bytes, MB/s e ETA em `JobOverview` | testes de modelo, React e carga |
| Diagnóstico exato | `job_metrics` por etapa | testes Rust e relatório exportável |
| Galeria progressiva | ativo catalogado antes de miniatura/réplica | pipeline integrado e testes de consolidação |
| Mesmo disco | aviso comparando volumes Windows | teste/renderização da Atividade |
| Segurança | SHA-256, temporário, fsync e verificação | testes de hash, colisão, cópia e backup |
