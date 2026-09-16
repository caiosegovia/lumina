# Especificação 0.22.2 Beta

## Objetivo

Estabilizar as jornadas já homologadas sem ampliar o produto. A versão corrige a classe de falhas que permitia crescimento de memória, trabalho sem dono e operações não canceláveis em galerias grandes.

## Entregas

- mídia e respostas internas limitadas, com `Range` validado e blocos de até 4 MiB;
- arquivos individuais limitados a 64 GiB antes de decoder, hash ou cópia;
- stdout e stderr de processos externos limitados a 8 MiB por canal;
- cache de miniaturas do frontend em LRU de 512 entradas;
- descarte explícito de players de vídeo ao trocar ou fechar a visualização;
- jobs com lease, heartbeat e recuperação persistidos no catálogo;
- filas de I/O e processos observam cancelamento enquanto aguardam;
- pressão de memória reduz concorrência e adia miniaturas de fundo;
- migração de acervo e exportação de eventos paginadas;
- snapshots SQLite consistentes em vez de cópia crua do banco;
- insights sob demanda, canceláveis e sem materializar o catálogo inteiro;
- métricas da árvore de processos, heartbeat do frontend e diagnóstico serializado;
- encerramento coordenado com cancelamento e espera limitada a dois segundos;
- CSP explícita e configuração crítica reconciliada com o catálogo.

## Compatibilidade

Windows 10/11 x64, mínimo de 8 GiB de RAM e quatro núcleos. Catálogos existentes são migrados automaticamente para o schema 20. A aplicação não altera os originais durante leitura, inventário ou geração de metadados.

## Fora do escopo

Novas funcionalidades, reconhecimento de pessoas, movimentação destrutiva de originais e promessa de produção antes do endurance humano. Os contratos completos estão em `NFR-0.22.2-RESILIENCE.md` e `TRACEABILITY-0.22.2.md`.
