# Rastreabilidade 0.6

| Requisito | Implementação | Evidência |
|---|---|---|
| Progresso compreensível | `stageText`, barra global e Central de trabalhos | testes React e smoke portátil |
| Controles de jobs | comandos persistentes de pausar, retomar e cancelar | testes `job_controls`, `cancel_removes`, `interrupted_analysis` |
| Galeria refinada | `ChoiceMenu`, grade/lista, agrupamento e densidade | build TypeScript e testes React |
| Desempenho | metadados em lotes, paginação por cursor, virtualização e cache versionado | testes de 2 mil/100 mil itens e thumbnails |
| Observabilidade | tabela `job_metrics` para tempo, itens e bytes | migração de catálogo v8 e testes Rust |
| Integridade | SHA-256, promoção atômica, colisão estável e validação | testes de pipeline, colisão e hash inválido |
| Portátil real | protocolo Tauri embutido e sinal de prontidão do React | `smoke-portable-0.6.ps1` |
