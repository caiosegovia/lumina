# Rastreabilidade 0.6.1

| Correção | Implementação | Validação |
|---|---|---|
| Falso travamento | progresso por lote em `capture_metadata_batches` | teste incremental e carga com 2.000 arquivos |
| Job preso ao cancelar | `mark_canceled` central e limpeza de estado intermediário | testes de cancelamento e processos externos |
| Atividade sem acabamento | `ActivityCenter.tsx` e design responsivo próprio | testes React e build TypeScript |
| Estados em inglês | mapa integral de estados para português | renderização da Central de Atividade |
| Pasta digitada manualmente | `chooseFolder` com diálogo nativo | teste React do botão e preenchimento |
| Executável sem localhost | build Tauri com protocolo interno | smoke do ZIP portátil |
