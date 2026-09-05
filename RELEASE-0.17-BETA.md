# Lumina 0.17.0-beta.1 — roteiro de homologação

## Atualização e integridade

1. Instale sobre a 0.16 e confirme que catálogo, favoritos, tags, álbuns e decisões permanecem.
2. Abra e feche o aplicativo três vezes; não deve haver crash, recuperação falsa ou job duplicado.
3. Confirme que nenhum original teve data, nome, conteúdo ou localização alterados.

## Atividade e metadados

4. Importe uma pasta grande e acompanhe o trabalho até `Pronto para revisar`.
5. Inicie a consolidação, pause, retome e deixe concluir.
6. Confirme que o job sai de `Em andamento`: proteção pendente deve estar em `Precisa da sua atenção`, não animada como execução.
7. Inicie `Completar metadados` enquanto outra importação estiver ativa.
8. Ao terminar a importação, confirme que metadados começam automaticamente e não ficam parados em `queued`.
9. Em Atividade, confirme que jobs técnicos `lumina://` não aparecem como importações.
10. Confira o painel `Organização em segundo plano`: Previews e Metadados devem mostrar fila, processamento, conclusão ou falha com contadores coerentes.
11. Pause um job: ele deve pedir `Retomar` ou `Cancelar`, sem animação ou atualização rápida de execução.

## Descobrir

12. Abra `Descobrir` e execute `Analisar biblioteca`.
13. Continue navegando durante a análise; a interface deve permanecer responsiva.
14. Ao concluir, compare o total analisado com o total indexável e observe indisponíveis/falhas.
15. Reinicie e confirme que o índice permanece, sem reanalisar itens já indexados.
16. Abra uma memória e confirme que a galeria encontra o arquivo correto.
17. Confira que memórias pertencem ao mês atual em anos anteriores.
18. Abra uma sequência e confirme que contém ao menos três registros do mesmo equipamento em intervalo curto.
19. Examine sugestões visualmente parecidas: devem ser plausíveis, mas nunca rotuladas como duplicatas.
20. Pesquise por características como `clara`, `escura`, `colorida`, `quente`, `fria`, `paisagem` e `retrato`.
21. Nas sequências, confira a marca `Melhor candidata` e valide se a recomendação técnica faz sentido visualmente.
22. Clique `Comparar`; os dois itens devem abrir selecionados na comparação da galeria.
23. Teste preview HD, zoom sincronizado, vídeo, metadados e proteção na comparação.

## Escala, falhas e regressão

24. Teste com catálogo grande e role todas as prateleiras; miniaturas devem carregar progressivamente.
25. Deixe uma origem offline e repita a análise: os arquivos indisponíveis devem ser contabilizados sem interromper os demais.
26. Valide Biblioteca em grade/lista, busca, filtros, seleção, favoritos, tags e álbuns.
27. Valide Duplicatas, Proteção, Revisão, Fontes e Visão Geral.
28. Exporte o diagnóstico depois do teste.

## Reprovação imediata

- Crash, tela branca ou congelamento persistente.
- Job técnico exibido como importação ou job concluído ainda animado.
- Metadados enfileirados que não iniciam após liberar o trabalho atual.
- Alteração de qualquer original.
- Similaridade tratada como duplicidade ou exclusão automática.
- Perda de organização pessoal ou índice após reiniciar.
