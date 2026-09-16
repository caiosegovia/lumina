# Arquitetura 0.22.1

## Pipeline de fotografia

```text
Galeria → miniatura já existente → prepare_photo_preview
        → permit exclusivo → FFmpeg isolado → JPEG ≤ 2560 px
        → lumina-preview (somente leitura) → WebView
```

O protocolo `lumina-media` é exclusivo para vídeos. O backend rejeita fotografias mesmo que uma regressão futura tente solicitar sua URL.

## Motor de insights

```text
InsightRequest (acervo/ano/mês; amostra/completa)
  → consulta SQLite
  → estratificação determinística
  → agregadores locais
  → cards acionáveis
  → insight_runs (cache versionado)
```

O módulo `insights` não depende do componente da galeria nem abre arquivos. A integração acontece por filtros serializáveis e rotas existentes.

## Localização v3.1

Campos nativos do arquivo formam um conjunto indivisível e têm precedência. A geocodificação offline só participa quando não existe nome embarcado. Ajustes manuais continuam acima das duas fontes.
