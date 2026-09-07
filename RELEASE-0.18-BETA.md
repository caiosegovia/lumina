# Lumina 0.18.0-beta.1 — roteiro de homologação

## Instalação e migração

1. Preserve a fonte original e instale a 0.18 sobre a 0.17.
2. Abra o catálogo existente e confirme contagens, favoritos, tags, álbuns e decisões.
3. Feche e reabra; confirme inicialização normal e ausência de trabalho duplicado.

## Estabilidade com aproximadamente 9.300 arquivos

4. Inicie **Completar metadados**.
5. Navegue continuamente por grade, lista, preview, zoom e vídeos durante o job.
6. Confirme que a interface segue responsiva e que o contador de metadados diminui.
7. Observe o consumo de memória no Gerenciador de Tarefas: crescimento temporário é aceitável; crescimento contínuo até travar não é.
8. Pause, aguarde e retome o trabalho.
9. Feche normalmente durante uma pausa, abra novamente e retome.
10. Deixe o enriquecimento terminar e confirme que não fica um job animado sem progresso.

## Proteção

11. Em Atividade, clique **Proteger agora** e confirme a mensagem de fila/início.
12. Confirme progresso na Atividade e redução da fila em Proteção.
13. Solicite proteção enquanto metadados estão ativos; ela deve aguardar e iniciar automaticamente depois.
14. Desconecte temporariamente a réplica e confirme erro acionável, sem marcar arquivos como protegidos.
15. Reconecte, repita e confirme verificação concluída.
16. Teste pouco espaço, pausa e retomada quando possível.

## Pessoas, tags e álbuns

17. Em Álbuns, crie uma pessoa.
18. Selecione mídias na galeria, use **Identificar pessoa** e confirme a associação.
19. Busque pelo nome e confira os resultados.
20. Remova a pessoa e confirme que as mídias permanecem intactas.
21. Aplique uma tag como `família > aniversário` e confira sua persistência.
22. Salve filtros como álbum inteligente, altere uma mídia e confirme a atualização do resultado.

## Lugares, viagens e descoberta

23. Abra Descobrir com mídias que tenham GPS.
24. Confira regiões aproximadas e ausência de resultados inventados para itens sem GPS.
25. Confira viagens somente para grupos de pelo menos três registros próximos.
26. Abra itens e comparação a partir dos novos grupos.

## Diagnóstico e crash

27. Em Atividade, abra **Diagnósticos e relatórios** e exporte o diagnóstico do aplicativo.
28. Confirme que o JSON contém `runtimeLogNewestFirst`, filas e versão 0.18.
29. Confirme que nomes de arquivos e caminhos pessoais não aparecem.
30. Se houver lentidão, travamento ou fechamento, reabra sem iniciar outro job e exporte imediatamente o diagnóstico.

## Aceite

Reprovação bloqueadora: crash; interface congelada; memória crescendo sem estabilizar; fila que não converge; proteção silenciosa; falso estado protegido; modificação de fonte; perda de organização; diagnóstico ausente após falha.

