# Lumina 0.24.1 beta 1

Pacote corretivo da homologação da 0.24.

- Corrige a seleção de arquivos no Explorer normalizando caminhos verbatim do Windows e abrindo a pasta correta como fallback.
- Impede o watchdog de redisparar jobs internos sem trabalho recuperável.
- Classifica falhas de preview sem incluir nomes ou caminhos no diagnóstico.
- Exibe causas recuperáveis e limitações permanentes na saúde das miniaturas.
- Gera miniaturas para vídeos com menos de um segundo a partir do primeiro frame.
- Redesenha o zoom do inspetor com passos de 25%, indicador, ajuste, pan, duplo clique e teclado.
- Separa memória do processo Rust, WebView2 e ferramentas externas nos logs.
- Não registra o aviso benigno do `ResizeObserver` como erro de aplicação.

Consulte `VALIDATION-0.24.1-BETA.md` antes da promoção para estável.
