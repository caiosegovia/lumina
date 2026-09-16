# Roteiro de homologação 0.22.2 Beta

Use uma cópia da galeria de homologação e guarde o diagnóstico ao final.

1. Instale a 0.22.2 sobre a 0.22.1 e confirme favoritos, tags, descrições, lugares manuais e estados de proteção.
2. Deixe a biblioteca de aproximadamente 9.300 itens aberta por 30 minutos sem interação; confirme que a memória estabiliza.
3. Navegue rapidamente por 1.000 fotos, abra e feche zoom e tela cheia, e confirme que a prévia acompanha a seleção.
4. Reproduza dez vídeos, avance a timeline e alterne entre foto e vídeo; confirme fluidez e ausência de áudio/processo residual.
5. Execute metadados, miniaturas e proteção; valide progresso a cada até cinco segundos e conclusão sem job “preso”.
6. Cancele um job enquanto ele processa e confirme resposta em até dois segundos, estado recuperável e interface responsiva.
7. Feche o Lumina durante um job, reabra e confirme retomada explícita sem duplicação ou dois escritores.
8. Gere insights por amostra, ano e mês; cancele uma execução e valide cache somente quando o catálogo não mudou.
9. Teste duplicatas, comparação, favoritos, notas, tags, bursts e “Mostrar arquivo no Explorador”.
10. Desconecte temporariamente uma fonte e a réplica; confirme erro explicável e nenhuma remoção do catálogo.
11. Abra uma mídia maior que 64 GiB, se houver fixture apropriada; confirme recusa antes de processamento.
12. Monitore a árvore `Lumina.exe`, FFmpeg e ExifTool: o conjunto não deve exceder 1,25 GiB nem manter filhos após encerrar.
13. Feche normalmente, confirme ausência de `running.session` e exporte os diagnósticos.

Reprove a versão se houver crash, crescimento contínuo de memória, interface sem heartbeat por mais de cinco segundos, job sem transição explicável, perda editorial, processo filho órfão ou modificação de original.
