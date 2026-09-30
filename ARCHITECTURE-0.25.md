# Arquitetura 0.25 — experiência da galeria

```text
preview limitado -> modelo geométrico puro -> palco responsivo
                   (fit/zoom/pan/clamp)       -> inspetor redimensionável

catálogo -> galeria virtualizada -> grade | lista -> seleção | inspeção | comparação

filas duráveis -> atividade do usuário
              -> manutenção silenciosa apenas enquanto acionável
```

O modelo de transformação não acessa DOM nem catálogo. Ele recebe viewport, dimensões naturais, escala, deslocamento e âncora; devolve uma transformação limitada. Isso permite testar responsividade e bordas sem WebView.

O `ResizeObserver` apenas mede o palco. Mudanças de layout recalculam o modo escolhido — ajustar, tamanho real ou preencher — enquanto zoom livre é limitado novamente ao novo viewport.

O painel integrado conserva largura no dispositivo, mas cai para painel flutuante em viewports estreitos. Nenhuma preferência contém caminho, metadado ou conteúdo pessoal.
