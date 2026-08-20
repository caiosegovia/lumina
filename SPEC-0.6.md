# Lumina 0.6 — especificação entregue

## Escopo

- Central de trabalhos separada em andamento, aguardando o usuário e histórico.
- Pausar, retomar, cancelar, revisar detalhes e repetir falhas, com estado persistido.
- Progresso global disponível durante a navegação, com linguagem orientada ao usuário.
- Controles visuais próprios para agrupamento por dia, mês ou ano e densidade da grade.
- Galeria em grade e lista, filtros combináveis, estatísticas e navegação paginada/virtualizada.
- Extração de metadados em lotes, cache de miniaturas e métricas persistentes de duração e volume.
- Cancelamento cooperativo, retomada idempotente e preservação absoluta das origens.
- Distribuição portátil com frontend embutido; nenhum servidor localhost é necessário.

## Regras de segurança

O Lumina não move, altera ou exclui arquivos nas origens. Cópias consolidadas são promovidas somente depois de validação e hash. Trabalhos cancelados preservam histórico e itens já verificados; temporários pertencentes ao trabalho podem ser descartados com segurança.

## Limites conhecidos

O paralelismo é limitado por recurso para não saturar discos externos. A extração externa usa o limitador global de processos; leitura e hash permanecem conservadores em mídia rotacional. O estado “réplica verificada” não afirma que o cliente do Google Drive concluiu o upload remoto.
