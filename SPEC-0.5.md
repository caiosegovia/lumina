# Lumina 0.5 — pacote combinado 0.4 + 0.5

## Escopo entregue

- Central persistente de trabalhos, indicador global, contador na navegação, reabertura de análises/importações e notificações internas.
- Análises adicionais entram em fila; somente um trabalho que escreve na biblioteca executa por vez.
- Galeria paginada por cursor, virtualizada, com cache LRU de miniaturas e índices SQLite já existentes.
- Visões em grade e lista, com preferência persistida; agrupamento por dia, mês ou ano e três densidades de grade.
- Big numbers de volume, espaço, proteção, localização, múltiplas origens e distribuição anual.
- Filtros combináveis, incluindo datas suspeitas.
- Seleção múltipla e ações em lote: tags, álbuns e correção auditável da data de captura.
- Criação de álbuns sem duplicar ou modificar arquivos originais.
- Miniaturas reais no cartão e preview, cache limitado e auditoria/reparo integral.

## Garantias

Arquivos de origem nunca são movidos, editados ou excluídos. Organização editorial existe no catálogo. Correções de data geram histórico em `asset_edits`. Operações em lote são transacionais e limitadas a 5.000 itens por chamada.
