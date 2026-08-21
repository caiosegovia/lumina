# Lumina 0.11 — entrega e validação

## O que mudou

- A Visão geral abre por snapshot/rollups e atualiza a análise completa em segundo plano.
- O snapshot é versionado, invalidado por alterações relevantes e pode ser reconstruído.
- A tela separa números, composição, linha do tempo, formatos/equipamentos, duplicidade, insights e diagnóstico.
- Formatos e anos abrem a galeria com o filtro correspondente.
- A duplicidade distingue espaço adicional conhecido de espaço conservador apto para futura revisão; nada é excluído.
- O inventário reconhece famílias ampliadas de fotos, RAW e vídeos, registra contêiner/codec, suporte e divergência de extensão.
- O enriquecimento do acervo existente é um job persistente, retomável, cancelável e de baixa prioridade.
- Formatos de preservação sem decoder continuam aceitos como mídia válida sem preview.

## Segurança

A migração cria apenas estruturas de catálogo. O inventário lê os arquivos consolidados e não move, edita ou exclui origens ou originais. A 0.11 não executa decisões de exclusão de duplicatas.

## Desempenho e diagnóstico

`dashboard_metrics` conserva as 100 medições mais recentes. A interface mostra tempos por seção em “Diagnóstico da visão geral”. O benchmark de release cria catálogos isolados de 100 mil e 500 mil registros, mede 20 leituras e mantém uma leitura concorrente do catálogo.

Resultado desta build: 100 mil itens, p50 2 ms/p95 4 ms; 500 mil itens, p50 3 ms/p95 4 ms. Em ambos os casos havia uma varredura concorrente ativa.

Comando:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml dashboard_rollups_scale_to_100k_and_500k_with_concurrent_reads -- --ignored --nocapture
```

## Roteiro de smoke test

1. Abra o executável e confirme que a Visão geral aparece antes de “Atualizando análises”.
2. Clique em Atualizar visão e confirme que a navegação continua responsiva.
3. Expanda Diagnóstico da visão geral e confira as durações.
4. Clique em um ano e em um formato; confirme os filtros na galeria.
5. Inicie Atualizar inventário técnico e acompanhe/controle o job em Atividade.
6. Confira grupos em Duplicatas: espaço adicional, estimativa conservadora e aviso de proteção.
7. Importe amostras JPEG, TIFF, HEIC/AVIF, DNG/CR2/CR3/NEF/ARW e vídeos MP4/MOV/MKV.
8. Confirme que um formato válido sem preview é preservado e não aparece como corrompido.

## Checklist dos 22 requisitos

| # | Resultado | Evidência |
|---|---|---|
| 1 | OK | Snapshot inicial + refresh assíncrono |
| 2 | OK | `DashboardTiming`, `dashboard_metrics` e diagnóstico |
| 3 | OK | `dashboard_snapshots`, schema version e reconstrução |
| 4 | OK | Triggers de assets, proteção, fontes, ocorrências, thumbs e inventário |
| 5 | OK | Snapshot permanece visível se o refresh falhar |
| 6 | OK | Cards com quantidade, bytes, período, pendências e ações |
| 7 | OK | Composição/linha do tempo e rankings |
| 8 | OK | Rollups por tipo, formato, ano, câmera e fonte |
| 9 | OK | Quantidade/bytes gerais; origem e período navegáveis |
| 10 | OK | Grupos e estimativas `additional`/`reclaimable` |
| 11 | OK | `occurrence_decisions`, sem operação destrutiva |
| 12 | OK | Painel independente de insights |
| 13 | OK | Causa, impacto, confiança, prioridade e ação |
| 14 | OK | Registro técnico de família, formato, contêiner e codec |
| 15 | OK | Assinaturas e `extension_matches` |
| 16 | OK | complete/partial/preservation/unknown/invalid |
| 17 | OK | `ValidWithoutPreview` coberto por teste |
| 18 | OK | `technical_enrichment` + `work_queue` |
| 19 | OK | Migração somente de catálogo e testes de segurança existentes |
| 20 | OK | Ano, formato e insight de datas abrem filtros contextuais |
| 21 | OK | Benchmark isolado 100k/500k sob leitura concorrente |
| 22 | OK | Suítes, documentação, portátil, EXE, MSI e smoke de release |
