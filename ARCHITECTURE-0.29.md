# Arquitetura — Lumina 0.29

## Fluxo da busca

```text
Campo de busca
  ├─ Enter / botão Buscar
  ├─ normalização (trim)
  └─ GalleryFilters.query
        └─ comando search_gallery
              ├─ parâmetros SQLite vinculados
              ├─ relações catalogadas (tags, álbuns e lugares)
              ├─ resumo agregado
              └─ página limitada + cursor opaco
                    └─ grade/lista virtualizada
```

O texto em edição é separado do filtro aplicado. Isso impede uma consulta por tecla e torna previsível quando o catálogo será consultado. O contador sequencial do frontend ignora uma resposta que chegue depois de outra consulta mais recente.

## Limites preservados

- A consulta não lê originais, previews ou metadados fora do catálogo.
- O limite recebido pelo backend é restringido ao intervalo de 1 a 200.
- IDs enviados por filtros são limitados a 5.000.
- Relações da página são carregadas em lote, sem N+1.
- Busca, filtros, resumo e paginação compartilham o mesmo predicado.

## Escopo deliberadamente inalterado

Importação, consolidação, proteção, filas duráveis, geração de previews, datas de captura, abertura no Explorer, duplicatas e Descobrir não foram reescritos nesta versão.
