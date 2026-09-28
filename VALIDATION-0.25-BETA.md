# Roteiro de homologação — 0.25 beta 1

## Feedback recebido em 26/09/2026 — visualizador reprovado

Durante a homologação da entrega 0.25.1-beta.1, o usuário relatou:

- Zoom continua ruim e não funciona, tanto em fotos quanto em vídeos.
- Responsividade muito ruim ao abrir uma foto na galeria.
- Barras desaparecem durante a interação com a mídia.

Este relato reprova o gate do visualizador, apesar dos testes automatizados anteriores. A causa técnica e quais barras desaparecem ainda precisam ser identificadas. Não implica novo resultado para setup, importação ou proteção.

Reteste após a correção: abrir e alternar fotos/vídeos, usar zoom e arraste, redimensionar painel/janela e alternar tela cheia. As barras devem permanecer acessíveis, a interface deve responder às ações e os controles de reprodução devem continuar funcionais. Incluir diferentes escalas do Windows e validação no executável instalado.

## Atualização

1. Instale sobre a 0.24.2 e confirme catálogo, favoritos, tags, álbuns, lugares e proteção.
2. Confirme que não existe reconstrução geral de previews nem alteração dos originais.

## Visualizador — gate obrigatório

3. Abra JPEG, RAW vertical, RAW horizontal e foto de celular.
4. Redimensione o inspetor arrastando o divisor e usando as setas quando ele estiver focado.
5. Em Ajustar, redimensione a janela: a fotografia deve permanecer inteira e centralizada.
6. Teste roda sem Ctrl nos quatro quadrantes; o ponto sob o cursor deve permanecer estável.
7. Arraste em todas as direções: nenhuma borda pode atravessar o viewport deixando área vazia indevida.
8. Teste Ajustar, 100%, Preencher, +, -, 0, duplo clique e 1200%.
9. Entre e saia da tela cheia ampliado; teste também escala do Windows em 100%, 125%, 150% e 200%.
10. Navegue por 200 fotos; cada nova foto deve iniciar ajustada e receber a prévia HD correta.

## Produtividade

11. Alterne grade/lista, densidade e ordenação; reinicie e valide as preferências.
12. Use pills de fotos, vídeos, RAW, favoritas, localização, revisão e sem proteção.
13. Selecione intervalos, aplique tags, álbuns, favoritos e revisão; confirme que não há ação de pessoas.
14. Redimensione o painel entre 340 e 620 px; em janela estreita ele deve virar painel flutuante sem cobrir controles essenciais.

## Atividades, duplicatas e insights

15. Execute metadados/previews: manutenção concluída deve desaparecer; pendência ou limitação deve continuar explicada.
16. Expanda duplicatas, compare ocorrências e aplique decisões em lote; nenhuma ação deve excluir arquivo.
17. Gere insight amostral e completo por ano e mês e abra o recorte resultante.
18. Recalcule lugares, renomeie um grupo e teste curadoria de burst; nada deve escrever no EXIF.

## Endurance

19. Navegue por 1.000 mídias durante processamento em segundo plano e mantenha o app aberto por 30 minutos.
20. Exporte o diagnóstico final e confirme ausência de crash, processo órfão, job preso ou crescimento contínuo de memória.

Reprovar por zoom imprevisível, área vazia além das bordas, preview trocado, perda editorial, job preso, crash ou modificação do original.
