# Evidências de validação — Lumina 0.15.0-beta.2

Data: 2026-09-02. Ambiente: Windows x64.

## Contratos automatizados novos

- Ordenação e paginação: cursores sem sobreposição para captura asc/desc e nome asc/desc.
- Prioridade: solicitação repetida mantém um único trabalho e eleva sua prioridade.
- Interface: ordenação enviada ao backend e persistida.
- Interface: densidade da lista persistida.
- Interface: viewport enviado em lote limitado com prioridade interativa.
- Interface: comparação disponível somente para duas mídias.
- Interface: seções de metadados e abertura de localização acionáveis.
- Interface: seleção possui destaque textual e visual em grade e lista.
- Interface: inspetor não apresenta faixa inferior e omite metadados sem valor de decisão.
- Visão geral: capacidade e proteção consolidadas, composição fotos/vídeos e comparação entre jobs.
- Catálogo: primeira/última captura calculadas separadamente para fotos e vídeos.

## Gates da candidata

- Frontend: 24/24 testes aprovados.
- Backend: 99 testes aprovados, 0 falhas e 2 opcionais ignorados.
- TypeScript e Vite: build de produção aprovado.
- Ordenação e cursor exercitados em integração com SQLite real.
- Catálogos de 100 mil itens, processamento de 2 mil arquivos e consultas concorrentes com miniaturas aprovados.
- A beta.1 homologada permanece disponível como rollback durante essa validação.

## Artefatos

- Smoke portátil: manifesto com 567 entradas validado, frontend pronto, encerramento limpo e 30.769.152 bytes de working set no teste.
- `Lumina-0.15.0-beta.2-portable-windows-x64.zip` — SHA-256 `93733a4463c58709cc8f99cfb3b1e6dd7faa3a210633544f598740ae8d40f530`
- `Lumina_0.15.0-2_x64_en-US.msi` — SHA-256 `94ec8a6075ed95cdabae48d49c927b51cafaf98aaee3f2cf8da3a3de249e29a0`
- `Lumina_0.15.0-2_x64-setup.exe` — SHA-256 `1ee1c40cfdd9a7dce557761d8ef4e1fbb2b187037cef39a0a49de961a3b4ddae`
