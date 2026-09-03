# Lumina 0.16.0-beta.1 — roteiro de homologação

## Atividade e importação

1. Faça uma importação completa e abra Atividade.
2. Durante cópia/proteção, confirme progresso e atualização fluida.
3. Ao chegar em “Proteção pendente”, confirme que a tela informa que os arquivos já estão no acervo e apresenta o próximo passo, sem animação de processamento.
4. Conclua a proteção e confirme que o job migra para Histórico.
5. Recolha o histórico e confirme que trabalhos encerrados deixam de dominar a tela.
6. Feche durante um trabalho, reabra e valide a retomada segura.
7. Teste réplica offline, falta de espaço, pausa, retomada e cancelamento.

## Duplicatas

8. Abra Duplicatas e confirme que os grupos aparecem como linhas compactas sem imagens grandes.
9. Confira pills de cópias, espaço adicional, proteção e decisão.
10. Teste os cinco filtros e as três ordenações.
11. Expanda um grupo e confirme que preview, origens, caminhos e decisões aparecem somente nesse momento.
12. Recolha o grupo e expanda outro; a lista deve continuar rápida e estável.
13. Marque manter, revisar e candidata; confirme persistência após reiniciar.
14. Confirme que “Candidata” permanece bloqueada sem réplica verificada.
15. Em mais de 50 grupos, use “Carregar mais” e confira continuidade sem duplicação.

## Regressão das seções

16. Visão Geral: valide números, datas, gráficos, insights e desempenho.
17. Biblioteca: valide grade/lista, filtros, ordenação, seleção, preview HD e comparação.
18. Revisão: abra cada fila e conclua ao menos uma pendência.
19. Fontes: desconecte/reconecte uma fonte e execute sincronização.
20. Álbuns/Tags: crie, renomeie, aplique e remova organização pessoal.
21. Proteção: execute verificação, reparo de previews e exportação de diagnóstico.
22. Reinicie o aplicativo e confirme persistência das decisões e preferências.

## Invariantes

- Fontes e originais não podem ser modificados.
- Nenhuma duplicata pode ser removida automaticamente.
- “Protegida” exige réplica verificada.
- Erros devem apresentar motivo e ação recuperável.

## Integridade dos artefatos

- Portátil: `00391e4231c1dba0bb3641e5a583f0a9f5e730caaaca22c50aa4ed01bbc4afe1`
- MSI: `f768e7f8e84e02bb9111eec3d12562f2a7fae1a4c2d84fe0912f2c21f83890df`
- Instalador EXE: `b1c0fdcc5f19e9b37d4cf663f599372ed98c978182e813eb5f71b1ad35ac8c68`
