# Lumina 0.9.1 — correção produtiva de RAW

- O pacote portátil deve conter a distribuição completa do ExifTool e provar que ela inicia após extração.
- Falhas de carregamento de DLL são classificadas como dependência indisponível, nunca como corrupção da mídia.
- A confirmação da importação chama itens recusados de “para revisar” e agrupa causa, extensão, quantidade e tamanho.
- Itens não validados permanecem fora da consolidação e as origens nunca são alteradas.
- A montagem acontece em staging e só é promovida depois das verificações, impedindo entregas parcialmente substituídas.

