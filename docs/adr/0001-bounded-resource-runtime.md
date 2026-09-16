# ADR-0001 — Runtime limitado e supervisionado

- Status: aceito para implementação na 0.22.2
- Data: 2026-09-16

## Contexto

Limites independentes não impediram que vídeo, processos externos, WebView e jobs somassem mais memória que o equipamento suporta. Threads destacadas e tokens voláteis também tornaram cancelamento e encerramento difíceis de provar.

## Decisão

O Lumina adotará um runtime único, limitado por construção:

- toda operação recebe identidade, prioridade, deadline, token e estimativa de custo;
- `ResourceGovernor` controla memória, I/O, processos e slots por classe;
- nenhuma entrada externa define diretamente uma alocação;
- serviços e workers pertencem a um supervisor e possuem shutdown/join;
- jobs duráveis são a fonte de verdade; estado em memória é descartável;
- adaptadores de mídia, processo, SQLite e filesystem obedecem aos mesmos limites;
- pressão de recursos reduz trabalho derivado antes de afetar a interação.

## Consequências

Haverá mais tipos e infraestrutura, mas menos semáforos locais, threads ad hoc e estados ambíguos. Funcionalidades existentes serão migradas incrementalmente por adaptadores, preservando o catálogo. A implementação exige testes de fairness, cancelamento, pressão e recuperação, além dos testes funcionais.

## Alternativas rejeitadas

- Corrigir apenas o `Range`: deixa stdout, caches e concorrência sem limite.
- Aumentar o limite de memória: falha em equipamentos mínimos e arquivos maiores.
- Um semáforo por feature: repete a fragmentação atual.
- Reescrita total: amplia risco e perde evidências das jornadas já homologadas.
