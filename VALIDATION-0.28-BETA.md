# Roteiro de homologação — 0.28.0-beta.1

## 1. Instalação e regressão básica

1. Instale por cima da 0.27 e confirme que biblioteca, caminhos, favoritos, tags e jobs permanecem.
2. Abra a galeria e navegue por fotos e vídeos durante 10 minutos.
3. Execute uma importação pequena e confirme que análise, consolidação e proteção continuam iguais à versão anterior.

## 2. Contraste, tema e tipografia

1. Alterne entre temas claro e escuro.
2. Verifique botões primários, secundários, menus, filtros e estados desabilitados.
3. Confirme que nenhum botão possui texto claro sobre fundo claro.
4. Confirme Inter na interface e foco visível ao navegar com `Tab`.

## 3. Galeria e seleção contextual

1. Confirme busca, ordenação, filtros e grade/lista na barra principal.
2. Abra “Mais opções” e teste agrupamento, densidade/tamanho e visão salva.
3. Selecione 1, 2 e 5 arquivos; confirme a barra contextual e o botão de cancelar seleção.
4. Com 2–4 arquivos, abra Comparar e conclua uma decisão.
5. Redimensione a janela até 1000×700 e confirme que comandos não somem nem se sobrepõem.

## 4. Tags

1. Selecione arquivos e abra Tags.
2. Pesquise e escolha uma tag existente.
3. Digite um nome inédito, use “Criar” e aplique junto com outra tag.
4. Confirme o total de arquivos, a mensagem de sucesso e “Desfazer”.
5. Abra Álbuns > Tags e confirme as classificações criadas.

## 5. Descobrir

1. Confirme somente Memórias, Lugares e Bursts para revisar.
2. Confirme que “Viagens”, “Visualmente parecidas” e “sessões de curadoria” não aparecem.
3. Use a busca e abra um período, lugar e burst na galeria.
4. Expanda “Estado e manutenção da descoberta”; valide cobertura, atualização e recálculo de lugares.

## Evidências automatizadas em 28/09/2026

- `npm test`: 58 testes aprovados em 9 arquivos.
- `npm run build`: TypeScript e Vite aprovados.
- `npm audit --audit-level=moderate`: nenhuma vulnerabilidade encontrada.
- Browser Edge: viewer responsivo aprovado em 1400×900, 1100×700 e 1000×700.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`: aprovados.
- `cargo test --lib -- --test-threads=1`: 149 aprovados, 2 ignorados e nenhuma falha.
- Smoke do executável empacotado: análise real de foto/vídeo/duplicata, consolidação, réplica verificada, comparação, undo, viewer e encerramento limpo aprovados; o original sintético permaneceu inalterado.
