# Relatório de validação — Lumina 0.9

- Testes Rust: 48 aprovados, 0 falhas.
- Testes React: 10 aprovados, 0 falhas.
- Catálogo: 100 mil mídias com paginação, filtros e rollups.
- Carga de análise: 2 mil arquivos sem perda de histórico.
- Deduplicação: tamanho apenas elimina candidatos; duplicatas continuam confirmadas por SHA-256.
- Cópia integrada: hash calculado durante escrita, temporário relido e promovido somente após verificação.
- Segurança: origem preservada nos testes de cancelamento, arquivo grande, colisão e pipeline completo.
- Build web: aprovado, 1.579 módulos.
- Auditoria npm: 0 vulnerabilidades.
- Build desktop release: aprovado (`lumina.exe`, perfil otimizado).
- Pacote portátil: 564 entradas conferidas pelo manifesto.
- Smoke test Windows em perfil limpo: janela nativa aberta, responsiva e frontend carregado sem servidor local.

O ganho real depende da distribuição de tamanhos. A telemetria do novo teste mostra `hashedBytes`, `deferredHashItems`, cache e tempos por etapa para comparação direta com o baseline de 84,3 GB da versão 0.8.
