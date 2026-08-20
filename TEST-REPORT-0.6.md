# Relatório de testes — Lumina 0.6

Este arquivo acompanha a distribuição. Os resultados finais são preenchidos pela execução dos gates da versão antes da publicação.

- Rust: 41 testes unitários e integrados, incluindo 2.000 arquivos, 100.000 registros, retomada, cancelamento, integridade, vídeo, HEIC e RAW.
- React: testes de navegação, filtros, importação e renderização da aplicação.
- Build: TypeScript/Vite e Tauri release com protocolo interno.
- Segurança de dependências: auditoria npm em severidade moderada.
- Smoke: manifesto SHA-256 integral, abertura a partir do ZIP extraído, janela responsiva e sinal explícito `Lumina Ready` do frontend.

Nenhum teste manipula ou remove arquivos de origem do usuário.
