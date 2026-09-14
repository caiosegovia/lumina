# Especificação — Lumina 0.20.0-beta.1

## Objetivo

Transformar coordenadas GPS já existentes em lugares úteis para busca e curadoria, mantendo todo o fluxo local, privado, explicável e reversível.

## Escopo entregue

1. **Resolução local:** associa mídias georreferenciadas a cidades conhecidas por um catálogo embarcado; coordenadas fora da cobertura continuam como regiões aproximadas claramente identificadas.
2. **Correção manual:** cada agrupamento pode receber um nome personalizado, persistido somente no catálogo Lumina.
3. **Descoberta:** exibe contagens de mídias com GPS, cidade identificada e regiões a revisar, além das ações de nomear, corrigir e abrir o conjunto na Galeria.
4. **Busca integrada:** nomes automáticos e personalizados passam a participar da busca textual da Galeria.
5. **Segurança:** o processamento não modifica EXIF, arquivos originais ou fontes e não envia coordenadas à rede.
6. **Compatibilidade:** migração de catálogo v16 preserva bibliotecas anteriores e a resolução pode ser repetida sem duplicar vínculos nem apagar correções.

## Limites explícitos

- A base embarcada inicial cobre capitais e destinos frequentes; lugares sem correspondência confiável são mostrados como região e podem ser nomeados pelo usuário.
- Não há geocodificação externa ou sincronização de localização nesta beta.
- Viagens continuam sendo sugestões por proximidade temporal de registros que possuem GPS; nenhuma viagem é inferida para arquivos sem coordenadas.

## Critérios de aceite

- Rodar **Nomear lugares** repetidamente produz o mesmo resultado e preserva nomes manuais.
- Buscar cidade ou nome manual retorna somente as mídias vinculadas ao lugar.
- Nenhum arquivo de origem muda em conteúdo, data ou metadados.
- Catálogos 0.19 abrem e migram para a versão 16 sem perda.
- Todos os gates de `VALIDATION-0.20-BETA.md` passam no commit da tag.
