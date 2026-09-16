# Lumina 0.22.1 Beta

## Objetivo

Eliminar a regressão de memória do inspetor, tornar os lugares mais honestos e separar insights editoriais do inventário técnico.

## Contratos

- Fotografias nunca usam a rota do original no WebView; seguem miniatura → preview JPEG limitado a 2560 px.
- `lumina-media` aceita somente vídeos e mantém leitura por intervalos.
- Insights consultam apenas o catálogo SQLite, nunca os originais.
- A amostra é estratificada por mês, tipo e equipamento, com até 12 itens por estrato.
- Análises completas são explícitas e podem ser recortadas por ano ou mês.
- Resultados são cacheados por escopo, modo, versão do algoritmo e impressão do catálogo.
- Localizações embarcadas não são misturadas com fragmentos inferidos pela base offline.

## Critérios de aceite

- Navegação de fotos não chama `get_media_url`.
- Preview permanece limitado e possui backpressure.
- Cache dos insights é isolado em schema 19.
- Lugar informa origem e confiança: exata, provável ou aproximada.
- Regressão automatizada, build, Clippy e smoke do executável aprovados.
