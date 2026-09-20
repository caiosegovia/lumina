# Roteiro de homologação — 0.24.1 beta 1

1. Em uma foto com acentos ou espaços no caminho, clique em **Mostrar arquivo no Explorador**. O arquivo deve ficar selecionado; no mínimo, a pasta correta deve abrir.
2. Deixe o app ocioso por um minuto. O diagnóstico não deve repetir `watchdog_dispatch job=_thumbnail_b`.
3. Abra **Proteção e armazenamento**. Falhas de preview devem aparecer agrupadas por causa e indicar se podem ser reprocessadas.
4. Execute o reparo. Limitações permanentes não devem ser confundidas com processamento pendente.
5. Teste preview de um vídeo com menos de um segundo e confirme a miniatura.
6. Abra uma foto e teste zoom nos botões em passos de 25%, `+`, `-`, `0`, duplo clique, `Ctrl` + roda e arraste.
7. Troque de foto ampliada: a nova foto deve abrir ajustada e centralizada.
8. Repita o zoom em tela cheia e confirme que as setas continuam navegando.
9. Execute proteção e verificação. Os jobs devem continuar avançando e retomando após reinício.
10. Exporte o diagnóstico e confira `schemaVersion: 5`, `thumbnailFailures`, `webview_working_set_bytes` e `tool_working_set_bytes`.

Aceite: nenhum crash; catálogo íntegro; originais inalterados; ausência do loop do job interno; Explorer no destino correto; zoom previsível; falhas de preview explicáveis.
