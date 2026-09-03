# Evidências de validação — Lumina 0.16.0-beta.1

Data: 2026-09-03. Ambiente: Windows x64.

## Gates

- Frontend: 26 testes aprovados em 4 arquivos.
- Backend: 99 testes aprovados, 0 falhas e 2 fixtures opcionais ignoradas.
- TypeScript/Vite: build de produção aprovado.
- Rustfmt: aprovado.
- Clippy para todos os targets com `-D warnings`: aprovado.
- npm audit com certificados do sistema: 0 vulnerabilidades.

## Contratos exercitados

- Estados pós-importação ficam em “Precisa de ação”, não em execução.
- Polling rápido ocorre somente enquanto há trabalho executável.
- Duplicatas iniciam recolhidas e os detalhes aparecem depois da expansão.
- Filtros, ordenação, pills, decisões e trava por proteção permanecem funcionais.
- Regressão da importação, galeria, fontes, álbuns, atividade e proteção aprovada.
- Backend preserva restart, cancelamento, filas duráveis, deduplicação, cópia verificada e fontes somente leitura.

## Smoke empacotado

- Manifesto: 567 entradas verificadas por SHA-256.
- Frontend pronto e responsivo em perfil `LOCALAPPDATA` isolado.
- Encerramento limpo e marcador de sessão removido.
- Working set observado: 28.524.544 bytes.

## Artefatos

- `Lumina-0.16.0-beta.1-portable-windows-x64.zip` — SHA-256 `00391e4231c1dba0bb3641e5a583f0a9f5e730caaaca22c50aa4ed01bbc4afe1`
- `Lumina_0.16.0-1_x64_en-US.msi` — SHA-256 `f768e7f8e84e02bb9111eec3d12562f2a7fae1a4c2d84fe0912f2c21f83890df`
- `Lumina_0.16.0-1_x64-setup.exe` — SHA-256 `b1c0fdcc5f19e9b37d4cf663f599372ed98c978182e813eb5f71b1ad35ac8c68`
