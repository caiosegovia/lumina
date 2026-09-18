# Especificacao 0.23.1 Fix Beta 1

## Objetivo

Corrigir a coerencia entre a data original da midia, sua organizacao fisica e sua apresentacao no Windows, alem de tornar a renovacao tipografica inequivoca.

## Entregas

- Datas EXIF locais, mesmo sem fuso, determinam corretamente as pastas `ano/mes`.
- Datas RFC 3339 com offset preservam o calendario local da captura.
- Data atual nao e mais usada silenciosamente quando a captura possui um formato valido sem fuso.
- Copias verificadas preservam a data de modificacao fisica do arquivo de origem.
- Data de captura, data fisica e data do job continuam conceitos separados no catalogo.
- Mostrar no Explorador envia `/select,<arquivo>` como uma unica expressao segura.
- Caminhos com espacos, virgulas, parenteses e acentos possuem regressao automatizada.
- Plus Jakarta Sans e Sora sao embarcadas localmente e substituem a combinacao anterior.

## Compatibilidade

O pacote nao movimenta automaticamente arquivos ja consolidados. Novas consolidacoes usam a regra corrigida. Uma auditoria/migracao de arquivos existentes deve ser sempre explicita e reversivel.
