# Lumina 0.19.0-beta.1 — roteiro de homologação

Use uma cópia de homologação e preserve as fontes originais. O aplicativo não deve escrever nelas.

## 1. Instalação e regressão

1. Instale o NSIS ou MSI da release e abra o catálogo usado na 0.18.5.
2. Confirme contagem, favoritos, tags, pessoas, álbuns e decisões anteriores.
3. Navegue por grade e lista, alterne filtros e densidade e confirme seleção destacada.
4. Abra fotos e vídeos; valide preview HD, zoom, tela cheia, navegação e metadados corretos para cada item.
5. Compare duas mídias e volte à posição anterior da galeria.

## 2. Atividade e processamento invisível

6. Inicie **Completar metadados** e continue navegando por pelo menos dez minutos.
7. Confirme progresso em Atividade e organização separada de metadados/previews em segundo plano.
8. Pause e retome. Uma pausa deliberada não pode aparecer como travamento.
9. Feche durante uma pausa, reabra e retome sem duplicar o trabalho.
10. Se um job ativo não atualizar por dois minutos, confirme o alerta amarelo e abra os detalhes.
11. Em uma falha reproduzível, use **Reprocessar falhas** e confirme a quantidade reenviada.

## 3. Duplicatas

12. Abra Duplicatas e confirme a lista compacta com quantidade, espaço, proteção e decisão.
13. Expanda um grupo e confirme que as ocorrências aparecem; recolha e expanda outro grupo.
14. Selecione vários grupos, marque **Revisar depois**, feche e reabra e confirme persistência.
15. Em grupo sem proteção verificada, confirme que marcar candidatas permanece bloqueado.
16. Gere o plano de limpeza e exporte o relatório; nenhum arquivo deve ser excluído.

## 4. Proteção

17. Abra Proteção e confira pendentes, em cópia, protegidos, falhas, cobertura e bytes restantes.
18. Clique **Proteger agora** e confirme mensagem imediata, contagem em cópia e atualização automática.
19. Navegue por fotos e vídeos durante a cópia; a interface deve continuar responsiva.
20. Desconecte a réplica em um ensaio controlado, confirme falha acionável, reconecte e repita.
21. Verifique a réplica e confirme que somente hashes validados aparecem como protegidos.

## 5. Estabilidade e diagnóstico

22. No catálogo de aproximadamente 9.300 arquivos, execute metadados, navegação e proteção concorrentes.
23. Observe memória: picos transitórios são aceitáveis; crescimento contínuo, congelamento ou fechamento reprovam a beta.
24. Exporte o diagnóstico em Atividade e confirme que ele não contém nomes de mídia ou caminhos pessoais.
25. Feche normalmente, reabra e confirme ausência de jobs fantasmas e de pedido para retomar cache.

## Bloqueadores

Crash, interface congelada, fila que não converge, ação sem feedback, fonte modificada, falso estado protegido, perda de organização, decisão parcial em lote ou diagnóstico ausente bloqueiam promoção.
