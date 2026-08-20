# Lumina 0.3 — Galeria em escala

## Entrega

- Filtros combináveis por texto, período, ano, tipo, câmera, fonte, pasta original, extensão, localização, tag, álbum e proteção.
- Indicadores do conjunto filtrado: quantidade, espaço, proteção, geolocalização, múltiplas origens e distribuição anual por quantidade e bytes.
- Paginação por cursor estável (`captured_at`, `id`), páginas limitadas a 200 ativos e grade virtualizada.
- Índices SQLite para cronologia e filtros, FTS5 trigram para nome/câmera e `PRAGMA optimize`.
- Debounce, descarte de respostas obsoletas, cache LRU de 300 miniaturas e restauração de filtros/rolagem ao retornar à galeria.
- Miniatura real no preview; geração sob demanda quando ausente ou inválida.
- Auditoria integral de miniaturas com verificação de existência, escopo, versão, decodificação e dimensões, seguida de reparo opcional.
- Caminhos de cache são restritos à pasta `.lumina/cache/thumbnails`; origens nunca são escritas ou excluídas.

## Limites conscientes

- A miniatura é entregue pelo comando interno restrito e mantida em cache de memória limitado. Um protocolo de streaming dedicado poderá substituir essa transferência em uma versão posterior, sem alterar o contrato de ativos.
- A versão continua pessoal, local e Windows. Similaridade visual, reconhecimento facial e Google Takeout permanecem fora do pacote.

## Aprovação

- Suítes Rust e React verdes.
- Build release e portátil aprovados em smoke test.
- Consulta completa e primeira página validadas com 100 mil ativos.
- Auditoria comprovada com miniaturas ausentes e corrompidas.
- Fontes do usuário preservadas durante testes e limpeza final.
