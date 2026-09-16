# Arquitetura 0.22.1

> **Adendo pós-homologação:** o desenho abaixo expressava a intenção, mas o contrato de vídeo não foi implementado de forma completa. Requisições sem `Range` e intervalos explícitos grandes podem carregar o arquivo inteiro. A versão foi reprovada para produção; consulte `ARCHITECTURE-AUDIT-0.22.2.md`.

## Pipeline de fotografia

```text
Galeria → miniatura já existente → prepare_photo_preview
        → permit exclusivo → FFmpeg isolado → JPEG ≤ 2560 px
        → lumina-preview (somente leitura) → WebView
```

O protocolo `lumina-media` é exclusivo para vídeos. O backend rejeita fotografias mesmo que uma regressão futura tente solicitar sua URL.

Essa separação de tipo está correta, porém não garante leitura limitada. A correção da 0.22.2 deve impor no máximo 4 MiB por resposta independentemente do cabeçalho recebido.

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
