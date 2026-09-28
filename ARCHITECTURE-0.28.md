# Arquitetura visual — 0.28

## Decisões

1. O design system é a única fonte de tipografia, cor, superfície, borda, foco e estados de controles.
2. Cores funcionais são consumidas por tokens semânticos; componentes não inferem contraste a partir do tema.
3. Ações permanentes ficam na barra da galeria; ações sobre itens substituem a barra quando existe seleção.
4. Comandos secundários usam overflow responsivo, preservando os comandos mais frequentes.
5. Tags possuem três papéis visuais distintos: rótulo, filtro selecionável e valor removível.
6. Descobrir apresenta conteúdo primeiro; manutenção e cobertura são detalhes progressivos.
7. A persistência antiga de curadoria permanece isolada e sem chamadas na jornada principal, evitando migração destrutiva.

## Fluxo de tags

`seleção na galeria -> abrir Tags -> pesquisar/selecionar/criar -> aplicar em lote -> confirmação -> desfazer opcional`

O frontend reutiliza `apply_tag` para cada nome escolhido. A operação continua registrada no histórico transacional do catálogo.

## Limites de regressão

- Nenhuma alteração em importação, consolidação, deduplicação, proteção ou promoção de arquivos.
- Nenhuma alteração no viewer e em seus limites de memória/backpressure.
- Nenhuma exclusão de schema ou dado legado.
- Falhas das chamadas de tags mantêm o diálogo aberto e exibem o erro.
