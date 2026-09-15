# Especificação — Lumina 0.21.0-beta.1

## Objetivo

Tornar datas, lugares e rajadas confiáveis para curadoria real, preservando os contratos de segurança, privacidade e responsividade das betas anteriores.

## Escopo entregue

1. **Relógio de captura:** datas EXIF sem fuso permanecem como o horário local escrito pela câmera. Timestamps operacionais e datas do sistema de arquivos continuam absolutos.
2. **Migração v17:** remove o falso UTC aplicado por versões anteriores a datas `exif_original` e `media_created`, sem reler nem modificar originais.
3. **Proveniência visível:** o inspetor informa se a data veio da câmera com fuso, da câmera como hora local, da mídia, do arquivo ou de correção humana.
4. **Lugares aprimorados:** usa primeiro a base mundial offline já distribuída com o ExifTool; a tabela local ampliada é fallback e usa raio conservador de 45 km.
5. **Filtro exato:** abrir lugar ou viagem usa a chave geográfica, não uma busca textual ambígua.
6. **Viagens legíveis:** quando resolvido, o destino participa do título e o período aparece no resumo.
7. **Bursts v2:** exige proximidade temporal, mesmo equipamento e, fora da janela imediata, coerência visual.
8. **Curadoria segura:** a melhor candidata pode ser favoritada e as alternativas enviadas para revisão; nenhum arquivo é excluído.

## Limites

- Data EXIF sem offset não permite descobrir historicamente o fuso real; o produto preserva o relógio da câmera e explicita essa incerteza.
- A base geográfica é offline. Um resultado sem confiança permanece aproximado e editável.
- “Manter melhor” organiza o catálogo; não remove nem oculta automaticamente originais.

## Critérios de aceite

- Uma foto EXIF `14:30` aparece como `14:30` independentemente do fuso configurado no computador.
- Atualizar um catálogo 0.20 preserva o horário de parede e toda organização anterior.
- Nomes manuais continuam soberanos após nova resolução.
- Bursts não misturam câmeras nem imagens visualmente incompatíveis fora da janela de dois segundos.
- Todos os gates de `VALIDATION-0.21-BETA.md` passam no commit da tag.
