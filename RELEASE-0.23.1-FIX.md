# Roteiro de homologacao 0.23.1 Fix Beta 1

1. Importe uma foto com `DateTimeOriginal` conhecido e sem offset; confirme a pasta fisica `ano/mes` da captura.
2. Importe uma foto de celular com offset; confirme que o dia, ano e mes nao sofreram conversao de fuso.
3. No Explorer, compare a data de modificacao da origem e da copia consolidada.
4. Confirme que a data de captura exibida no Lumina continua igual ao EXIF.
5. Use Mostrar arquivo no Explorador em um nome simples e confirme o highlight.
6. Repita com nome contendo espaco, virgula, parenteses e acentos.
7. Confirme visualmente Plus Jakarta Sans em textos, controles e navegacao.
8. Confirme Sora em titulos e numeros de destaque.
9. Repita a revisao tipografica nos temas Claro e Escuro.
10. Execute uma importacao, feche e reabra; confirme jobs, catalogo e originais intactos.

Arquivos consolidados por versoes anteriores nao sao movidos por este pacote.
