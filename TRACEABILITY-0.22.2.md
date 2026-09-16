# Matriz de rastreabilidade — 0.22.2

| Jornada / risco | Requisitos | Achados | Evidência obrigatória |
|---|---|---|---|
| Abrir/buscar vídeo grande | CAP-04/05, MEM-03/05/07, RES-01 | ARC-001/003 | Protocol tests + E2E com arquivo esparso de 64 GiB; pico < 800 MiB |
| Extrair metadados hostis | MEM-04/05/08, CAN-01, RES-01 | ARC-002/013 | Filho com saída infinita; truncamento, cancelamento < 2 s e árvore encerrada |
| Navegar 500 mídias | MEM-02/06, LAT-01/02/04 | ARC-017/018/019/020/034 | E2E no executável; residual <= 100 MiB e heartbeat <= 5 s |
| Scroll de 100 mil itens | CAP-03, MEM-02, LAT-02 | ARC-017/018 | 100 mil no catálogo, windowing de dados e p95 documentado |
| Rodar job durante galeria | MEM-04/05, LAT-01/04, CAN-01 | ARC-006/010/013/030 | Soak concorrente com budgets e prioridade interativa |
| Cancelar/fechar/reabrir | CAN-01/02/03, DAT-06, RES-04/05 | ARC-009/010/011 | Fault injection em cada estágio, nenhum job eterno, retomada idempotente |
| Alterar réplica | DAT-03/04/06 | ARC-004/022/038 | Falha injetada antes/depois de cada commit; estado antigo ou novo, nunca híbrido |
| Migrar acervo | DAT-03/05/06, MEM-04 | ARC-005/021/025 | Backup API sob WAL ativo, kill/restart, catálogo íntegro e memória limitada |
| Gerar insights | MEM-04, CAN-01, DAT-03 | ARC-014/015/016/029 | Duas solicitações concorrentes, cancelamento isolado, invalidação por domínio |
| Exportar diagnóstico | OBS-01..05 | ARC-007/008/024 | Fixture hostil de PII, logs concorrentes, rotação e árvore de processos |
| Segurança do WebView | SEC-01 | ARC-001/023 | CSP testada, capabilities mínimas e fuzz de URI/range |
| Distribuir beta | MAN-03 e todos os budgets | ARC-034/035/036/037 | CI, instaladores, smoke funcional, soak e relatório assinados pelo commit |

## Cenários mínimos do gate

1. Instalação limpa sem catálogo.
2. Upgrade da 0.22.1 preservando favoritos, tags, correções, lugares e jobs.
3. Catálogo de 100 mil registros e galeria de homologação com mídias reais.
4. JPEG/RAW extremos, arquivo inválido, vídeo 0 B, vídeo normal, 64 GiB esparso e 64 GiB + 1 B.
5. Falta de espaço, fonte offline, réplica offline, ferramenta ausente, timeout e saída excessiva.
6. Kill do processo em descoberta, hash, validação, cópia, promoção, proteção e migração.
7. 2 h de uso misto no equipamento mínimo.

## Critério de decisão

- Qualquer P0 aberto: não gerar beta.
- Qualquer NFR de integridade/privacidade sem evidência: não gerar beta.
- Orçamento excedido de forma reproduzível: corrigir antes de release.
- Flake: teste não conta como evidência até a causa ser eliminada.
