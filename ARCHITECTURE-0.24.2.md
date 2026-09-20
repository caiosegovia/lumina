# Arquitetura corretiva 0.24.2

O pacote mantém a arquitetura local-first e a separação entre catálogo persistente, fila durável, ferramentas isoladas e WebView.

```text
catálogo SQLite -> fila seletiva v3 -> ExifTool (4 tags RAW)
                                      -> FFmpeg isolado -> cache derivado
                                      -> estado terminal + diagnóstico

catálogo -> comando Explorer -> Shell nativo -> selected | folder_opened

preview limitado -> WebView -> transformação ancorada e pan limitado
```

Invariantes:

- arquivos originais e datas de captura nunca são escritos pelo pipeline de preview;
- previews prontos são promovidos de versão, não regenerados;
- somente falhas de formatos potencialmente recuperáveis voltam à fila;
- falha permanente não mantém atividade em execução;
- subprocessos têm limite de tempo, tamanho e cancelamento;
- respostas de integração com o sistema operacional representam o resultado real.
