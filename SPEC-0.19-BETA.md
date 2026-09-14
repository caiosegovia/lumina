# Especificação — Lumina 0.19.0-beta.1

## Objetivo

Fechar o ciclo operacional entre importar, acompanhar, curar e proteger uma biblioteca grande sem reintroduzir bloqueios ou decisões técnicas na abertura do aplicativo.

## Escopo entregue

1. **Atividade:** estados ativos, de atenção e históricos permanecem separados; jobs ativos sem atualização por dois minutos recebem alerta distinto de uma espera normal na fila; falhas podem ser reenviadas; ações e próxima etapa ficam explícitas.
2. **Duplicatas:** o resumo inicial não carrega ocorrências; detalhes são consultados ao expandir; grupos podem ser selecionados e decididos em lote; candidatas continuam condicionadas a réplica verificada.
3. **Galeria e inspeção:** consolida como contrato da beta a grade/lista virtualizada, segmentação fixa, seleção destacada, preview HD progressivo, zoom, tela cheia, vídeo, metadados organizados e comparação.
4. **Processamento invisível:** metadados e previews continuam em fila durável, limitados e reconstruíveis, apresentados separadamente do trabalho iniciado pelo usuário.
5. **Proteção:** painel mostra quatro estados da fila, volume pendente, cobertura e atualização automática enquanto copia.

## Fora de escopo

- Exclusão física de originais ou ocorrências.
- Sincronização em nuvem e reconhecimento remoto.
- Decisões automáticas irreversíveis.

## Critérios de aceite

- Nenhuma consulta de abertura de Duplicatas executa uma consulta de ocorrências por grupo.
- Uma decisão em lote é integral: falha de segurança não deixa decisões parciais.
- Pausa não é rotulada como travamento; fila não é rotulada como execução travada.
- Proteção nunca altera o estado para verificado antes de copiar e validar o hash.
- Navegação e inspeção permanecem responsivas durante metadados, miniaturas ou proteção.
- Todos os gates descritos em `VALIDATION-0.19-BETA.md` passam no mesmo commit da tag.
