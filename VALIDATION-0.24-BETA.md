# Validação da 0.24 beta

## Automática

- Frontend: testes Vitest, TypeScript e build Vite.
- Backend: testes Rust, `cargo clippy` e build Tauri.
- Empacotamento: MSI e NSIS; smoke test do executável instalado/portátil.

## Homologação funcional

1. Abra a biblioteca de homologação e deixe Atividades visível durante uma proteção grande.
2. Confirme avanço de bytes e arquivos. Uma cópia longa com item em processamento deve aparecer como ativa, não como travada.
3. Ao terminar um job, confirme que o próximo trabalho persistido inicia em até 10 segundos, sem fechar o Lumina.
4. Feche o app durante uma etapa, reabra e retome. Não deve haver cópia duplicada ou estado `processing` abandonado.
5. Na galeria, abra JPEG, RAW e vídeo. A foto deve ganhar preview de alta qualidade sob demanda; RAW incompatível não deve gerar uma sequência de falhas FFmpeg.
6. Selecione uma mídia e use **Mostrar arquivo no Explorador**. A pasta correta deve abrir com o arquivo destacado.
7. Exporte o diagnóstico. Confira nome em hora local e campos `generatedAtUtc`, `generatedAtLocal`, `utcOffset` e `timezone`.
8. Verifique favoritos, tags, filtros, duplicatas, dark mode e reprodução de vídeo para regressão.

## Critérios de aceite

- Zero encerramento inesperado no ensaio prolongado com cerca de 9.300 itens.
- Zero fila pendente ociosa por mais de 10 segundos quando há capacidade e destino disponível.
- Zero falso alerta de travamento durante cópia comprovadamente ativa.
- Arquivo correto selecionado no Explorer.
- Catálogo íntegro e originais inalterados.
