# Lumina 0.29.0-beta.1 — busca e produtividade da galeria

## Resultado

A busca da galeria deixa de parecer um campo passivo: agora possui ação visível, contrato explícito por teclado, progresso, resultado e limpeza. A entrega preserva os fluxos homologados de importação, proteção, jobs, datas, Explorer e visualização da 0.28.

## Entregas

- Botão `Buscar`, submissão por `Enter`, foco por `Ctrl+F` e limpeza por `Esc` ou ação visível.
- Consulta somente após confirmação; digitar não dispara sucessivas operações no catálogo.
- Estado `Buscando…`, contagem de resultados e mensagem vazia contextual.
- Pills individuais para busca e filtros ativos, com remoção isolada e `Limpar tudo`.
- Busca catalogada por nome de arquivo, equipamento, tag, álbum e lugar resolvido/manual.
- Filtros existentes de foto, vídeo, RAW, favorito, período, fonte, pasta, extensão, localização, proteção, avaliação e revisão continuam combináveis.
- Barra adaptada para larguras de 1000 px sem esconder a ação principal.

## Compatibilidade e segurança

- Nenhum arquivo físico é aberto para responder à busca.
- Parâmetros continuam vinculados no SQLite; texto de usuário não é interpolado em SQL.
- Paginação por cursor e limite máximo de 200 registros por página permanecem ativos.
- Respostas obsoletas são descartadas pela sequência já existente no frontend.
- Não há alteração de schema, migração, pipeline de mídia ou contrato dos jobs.

## Artefatos

- `Lumina_0.29.0-1_x64-setup.exe`
- `Lumina_0.29.0-1_x64_en-US.msi`

## Integridade

- EXE SHA-256: `4F58DF68E4DF2F72A12257D9EC1E37F3FE06AB6A85262F3E29857FA382A9ED04`
- MSI SHA-256: `FAEA07EC6B6ABAEC867449750F5C7827AF0A079D859D2B986F9E914D38E65CD5`
