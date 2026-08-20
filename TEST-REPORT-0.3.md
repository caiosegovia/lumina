# Relatório de testes — Lumina 0.3

Executado em Windows x64 em 17/08/2026.

- Rust: 39 testes aprovados.
- React/TypeScript: 8 testes aprovados.
- Catálogo sintético: consulta filtrada, agregações e primeira página sobre 100 mil ativos dentro do orçamento de cinco segundos do gate.
- Mídia real: JPEG, HEIC, DNG/RAW e MP4.
- Miniaturas: ausência, corrupção, versão, escopo e reconstrução integral validados.
- Preview: teste confirma uso da miniatura real no painel de detalhes.
- Segurança de dependências: `npm audit` com zero vulnerabilidades conhecidas.
- Build: frontend, Rust release, MSI e NSIS produzidos.
- Portátil: 564 entradas verificadas por SHA-256; iniciou com `PATH` reduzido e ferramentas embarcadas.
- Reset pós-teste: 930 itens consolidados, catálogo, cache e configuração local removidos. Origens `D:\Fotos` e `D:\Teste Org` e backup `D:\Backup Galeria DRIVE` confirmados como preservados.
