# Lumina 0.21.0-beta.1 — roteiro de homologação

## 1. Migração e regressão

1. Instale sobre a 0.20 e abra o catálogo de homologação.
2. Confira contagens, favoritos, tags, álbuns, pessoas, duplicatas, proteção e histórico de jobs.
3. Navegue durante metadados e previews por dez minutos; travamento, crescimento contínuo de memória ou job fantasma reprovam.

## 2. Datas e horas

4. Escolha fotos cujo relógio você conheça e compare com `DateTimeOriginal` no ExifTool/câmera.
5. Confirme que `14:30` no EXIF aparece como `14:30` no Lumina, sem deslocamento de três horas.
6. Confira a pill de proveniência: **Câmera · fuso informado**, **Câmera · horário local**, **Criação da mídia**, **Data do arquivo** ou **Data corrigida**.
7. Teste uma foto próxima da meia-noite e confirme que ela não mudou de dia.
8. Corrija uma data em lote, feche e reabra; a correção deve persistir e ser identificada.
9. Ordene crescente/decrescente e agrupe por dia/mês/ano; a posição deve corresponder ao horário exibido.

## 3. Lugares e viagens

10. Em Descoberta, execute **Nomear lugares** sem internet.
11. Compare pelo menos dez resultados com as coordenadas conhecidas; resultados incertos devem aparecer aproximados.
12. Renomeie uma região, reprocesse e confirme que o nome manual permanece.
13. Clique **Ver na galeria** e confirme correspondência exata do agrupamento.
14. Confira títulos e períodos das viagens sugeridas.

## 4. Bursts

15. Confirme que uma rajada real aparece em **Bursts** e que imagens de outra câmera não entram nela.
16. Confira destaque e justificativa da melhor candidata.
17. Compare candidatas e valide nitidez/exposição visualmente.
18. Clique **Manter melhor**: a indicada deve virar favorita e as demais entrar em **Revisar depois**.
19. Reverta essas marcações na Galeria e confirme que nenhum arquivo foi movido ou excluído.

## 5. Operação

20. Execute proteção, navegação, lugares e bursts no catálogo de aproximadamente 9.300 arquivos.
21. Exporte diagnóstico; GPS, caminhos, nomes e hashes pessoais não podem aparecer.
22. Feche e reabra normalmente; filas devem convergir e o cache não deve pedir retomada indevida.

## Bloqueadores

Crash, congelamento, mudança de dia/hora indevida, cidade apresentada com falsa precisão, perda de override, burst misturado, decisão irreversível, fonte alterada ou job preso bloqueiam promoção.
