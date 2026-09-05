# Especificação — Lumina 0.17.0-beta.1

## Objetivo

Transformar o catálogo em uma experiência de descoberta e curadoria local, mantendo originais somente leitura, decisões humanas e processamento explicável. A entrega também fecha a ambiguidade operacional observada na 0.16.

## Escopo funcional

### Atividade sem falsos trabalhos presos

- Atividade lista apenas importações, sincronizações, proteção e verificações iniciadas pelo usuário.
- Filas reconstruíveis com origem `lumina://` não aparecem como jobs comuns.
- Previews e metadados têm um resumo próprio, discreto, com estados `Na fila`, `Processando`, `Atualizado` e `Requer revisão`.
- Contadores são derivados da fila durável; um job técnico permanente não representa execução ativa.

### Descoberta local

- Nova seção `Descobrir`, integrada à navegação principal.
- Índice perceptual dHash de 64 bits para fotos e RAW decodificáveis, armazenado no catálogo.
- Busca de candidatos por quatro bandas indexadas, evitando comparação quadrática integral.
- Similaridade é apresentada como probabilidade de variação, nunca como duplicidade exata.
- Índice é versionado, incremental e reconstruível; arquivos indisponíveis ou incompatíveis são contabilizados sem bloquear o catálogo.
- Características pesquisáveis incluem orientação, luminosidade, intensidade de cor e temperatura dominante; elas não fingem reconhecer objetos ou pessoas.

### Sequências

- Registros do mesmo equipamento separados por até 15 segundos formam uma sequência a partir de três itens.
- Grupos são ordenados por tamanho e limitados na interface para preservar responsividade.
- A regra é determinística, local e explicada ao usuário.
- O melhor candidato é sugerido por resolução, contraste/nitidez aproximada e equilíbrio de luminosidade; a recomendação não substitui avaliação humana.

### Memórias

- O período atual é relacionado a registros do mesmo mês em anos anteriores.
- Memórias navegam diretamente para o item no catálogo.
- Ausência de resultados é tratada como estado natural, não como erro.

### Comparação e curadoria

- Qualquer grupo com dois ou mais itens abre a comparação existente já selecionada.
- Comparação mantém previews HD, zoom sincronizado, metadados, proteção e hash.
- Nenhuma sugestão move, edita ou exclui originais.

## Invariantes

- Fontes e originais continuam somente leitura.
- Similaridade nunca altera o estado de duplicatas.
- Nenhuma exclusão automática existe.
- Índices derivados podem ser apagados e reconstruídos sem perda de organização pessoal.
- Uma falha de decodificação não impede as demais imagens de serem indexadas.

## Critérios de saída

- Ciclo de jobs técnicos e do usuário distinguível em banco e interface.
- Similaridade validada com imagens reais controladas.
- Descobertas abrem mídia e comparação corretamente.
- Testes de frontend/backend, build, Rustfmt, Clippy, auditoria e smoke empacotado aprovados.
- Instalador, MSI, portátil, hashes, documentação e roteiro publicados juntos.
