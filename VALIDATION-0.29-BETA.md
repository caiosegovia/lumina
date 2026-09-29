# Roteiro de homologação — 0.29.0-beta.1

## 1. Instalação e regressão

1. Instale por cima da 0.28 e confirme biblioteca, fontes, caminhos, favoritos e tags.
2. Abra fotos e vídeos, navegue, use zoom e selecione o arquivo no Explorer.
3. Confirme que Atividades e Proteção mantêm o estado anterior.

## 2. Ação de busca

1. Abra Biblioteca e digite parte de um nome. Confirme que a busca só acontece ao pressionar `Enter` ou `Buscar`.
2. Repita por equipamento, tag, nome de álbum e nome de lugar.
3. Pressione `Ctrl+F` e confirme que o campo recebe foco.
4. Pressione `Esc` dentro do campo e use o botão de limpar; ambos devem restaurar a galeria.
5. Durante a consulta, confirme `Buscando…`; ao terminar, confira a contagem.

## 3. Pills e combinações

1. Combine um texto com Fotos/Vídeos, ano, favorito e um filtro avançado.
2. Remova uma pill por vez e confirme que somente aquele critério deixa de valer.
3. Use `Limpar tudo` e confirme o retorno ao acervo completo.
4. Faça uma consulta sem correspondência e valide a orientação e a ação de limpeza.

## 4. Responsividade e estabilidade

1. Teste tema claro e escuro em 1400×900, 1100×700 e 1000×700.
2. Alterne grade/lista e abra o inspetor com busca e filtros ativos.
3. Faça 30 buscas consecutivas e navegue pelos resultados por pelo menos 10 minutos.
4. Mantenha um job em execução e confirme que a galeria continua responsiva.

## Evidências automatizadas

- Frontend: 59 testes aprovados em 9 arquivos.
- TypeScript/Vite: build aprovado.
- Rust: suíte integral aprovada com 150 testes, 2 ignorados e nenhuma falha.
- Testes específicos cobrem ação explícita, `Enter`, `Ctrl+F`, limpeza, parâmetros SQL, álbuns, lugares e paginação.
