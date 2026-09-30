# Lumina 0.28.0-beta.1 — revisão integral de UX

## Resultado

A 0.28 reorganiza a experiência sem alterar o contrato de preservação dos originais, importação, proteção, catálogo ou jobs homologados na 0.27.

## Entregas

- Inter Variable incorporada ao instalador e aplicada à interface inteira.
- Paleta semântica para temas claro/escuro; ação primária usa foreground próprio e legível em cada tema.
- Foco visível, estados de hover/pressionado/desabilitado e superfícies corrigidos.
- Barra da galeria reduzida a busca, ordenação, visualização, filtros e overflow.
- Barra contextual de seleção com Tags, Álbum, Favoritar, Comparar e ações secundárias agrupadas.
- Novo seletor de tags: pesquisa tags existentes, permite selecionar várias e cria uma nova por ação explícita.
- Sessões de curadoria removidas da interface; “Revisar depois” permanece como jornada direta.
- Descobrir simplificado para Memórias, Lugares e Bursts para revisar.
- “Viagens” e “Visualmente parecidas” removidas da experiência principal.
- Estado do índice, atualização e recálculo de lugares agrupados em manutenção sob demanda.

## Compatibilidade e segurança

- Tabelas e APIs antigas de curadoria permanecem nesta versão para migração/rollback sem perda de dados, mas não são carregadas pela galeria.
- O contrato Rust e o schema SQLite não mudaram.
- Nenhuma operação nova move, edita ou exclui arquivos originais.
- O processamento já homologado continua disponível; a mudança é de apresentação e prioridade das jornadas.

## Artefatos

- `Lumina_0.28.0-1_x64-setup.exe`
- `Lumina_0.28.0-1_x64_en-US.msi`
