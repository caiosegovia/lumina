# Lumina 0.24.2 beta 1

Pacote corretivo dirigido pelas evidências do diagnóstico 0.24.1.

- Miniaturas RAW tentam, em ordem, `PreviewImage`, `JpgFromRaw`, `OtherImage` e `ThumbnailImage`; a decodificação continua isolada no FFmpeg.
- A migração v3 preserva previews saudáveis e reprocessa seletivamente RAW e vídeos que falharam na versão anterior.
- O job interno de miniaturas termina quando não existe trabalho pendente e informa limitações separadamente.
- O zoom aceita roda sem tecla modificadora, mantém o ponto sob o cursor, limita o arraste e oferece Ajustar, 100% e Preencher até 800%.
- Mensagens de capacidade usam B/KB/MB/GB/TB em vez de bytes crus.
- “Mostrar arquivo no Explorador” informa honestamente se o Windows selecionou o arquivo ou apenas abriu a pasta correta.
- A promoção de camada do WebView fica restrita à imagem que está ampliada.

Nenhum original é alterado. O cache de miniaturas é derivado e reconstruível.
