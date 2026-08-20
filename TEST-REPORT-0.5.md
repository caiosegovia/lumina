# Relatório de testes — Lumina 0.5

Executado em 17/08/2026 no Windows x64.

- Rust: 41 aprovados, 0 falhas. Inclui pipeline real, corrupção, cancelamento, retomada, 100 mil registros, 2 mil arquivos, vídeos grandes, HEIC/RAW, cache e fila.
- React: 10 aprovados, 0 falhas. Inclui importação, recuperação, progresso, galeria, preview real, filtros, lista persistente e tag em lote.
- TypeScript + Vite: build de produção aprovado.
- `npm audit --audit-level=moderate`: 0 vulnerabilidades.
- Release Rust otimizado: aprovado.
- Smoke portátil corrigido: 564 arquivos conferidos; exige `MainWindowHandle` não nulo, título `Lumina` e janela responsiva. Aprovado também com o perfil real do usuário.
