# Especificação — Lumina 0.15 beta

## Objetivo

Transformar a galeria em um workspace único de descoberta e revisão. Grade, lista, segmentação e inspeção devem coexistir sem sobreposição, perda de contexto ou navegação desnecessária.

## Contrato funcional

- A segmentação principal permanece visível durante a navegação e reúne resumo, mídia, ano, busca, agrupamento, densidade, visões e filtros.
- Grade e lista compartilham filtros, seleção, agrupamento, paginação e posição da sessão.
- A lista apresenta identidade, captura, arquivo, origem e proteção com colunas previsíveis.
- Abrir uma mídia reorganiza o workspace e exibe um inspetor embutido; a galeria continua visível e acionável.
- Em largura reduzida, o inspetor vira uma superfície flutuante controlada sem quebrar a galeria.
- A sequência do visualizador é identificada, recolhível e tem preferência persistente.
- Visualização, filtros e densidade continuam persistentes entre entradas na galeria.

## Inegociáveis

- Nenhuma escrita ou remoção nas fontes.
- Nenhuma regressão no preview HD, vídeo, EXIF, favoritos, tags, álbuns ou proteção.
- Navegação por teclado, foco visível e semântica acessível.
- Virtualização e paginação preservadas para catálogos grandes.
- Estados vazios, carregamento e erro compreensíveis.
- Build, testes, Clippy, auditoria e smoke do aplicativo empacotado antes da homologação.

## Próximos incrementos do ciclo

- Ordenação persistente e cabeçalhos interativos.
- Densidade específica para a lista.
- Seleção por intervalo e seleção do resultado carregado.
- Painel de metadados organizado por seções recolhíveis.
- Comparação lado a lado acionável a partir da galeria.
- Processamento de miniaturas priorizado pelo viewport e totalmente silencioso.
