# Lumina 0.8 — armazenamento, desempenho e filas independentes

## Escopo entregue

1. Planejamento por seleção, margem operacional, mestre, réplica e unidade compartilhada.
2. Seleção parcial por tudo, tipo, ano, pasta e limite seguro de bytes; lotes restantes persistem.
3. Consolidação termina após verificar o mestre; proteção usa fila persistente separada.
4. Troca da réplica e migração do mestre por cópia verificada, sem remover o acervo anterior.
5. SHA-256 em pool adaptativo e limitado, buffer de 8 MiB, cache de arquivos inalterados e métricas.
6. Inventário rápido persistido antes da confirmação profunda de conteúdo.
7. Estados e controles independentes para análise, consolidação e proteção, recuperáveis após reinício.
8. Galeria disponibilizada após o mestre, com proteção pendente/erro e reparo de miniaturas.
9. Contadores separados, decisões registradas e formatos não suportados distintos de corrupção.

## Garantias

- Fontes nunca são movidas, alteradas ou excluídas.
- Promoção do mestre e da réplica ocorre somente após SHA-256.
- Seleções e filas são idempotentes e persistidas no SQLite.
- Migração não exclui o mestre anterior e troca o catálogo somente após verificação integral.
- Upload remoto do Google Drive continua fora do estado verificável do Lumina.
