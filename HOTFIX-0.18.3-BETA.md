# Lumina 0.18.3-beta.1

Correção baseada no diagnóstico exportado em 8 de setembro de 2026 após a 0.18.2.

## O que o relatório permite concluir

O catálogo permaneceu íntegro, com 9.269 arquivos, 9.207 miniaturas prontas e 62 falhas. Não houve `panic` Rust, falta de memória ou perda prolongada do heartbeat na janela disponível. A fila total acima de 12 mil incluía trabalhos diferentes — proteção, verificação e miniaturas — e não representa 12 mil miniaturas duplicadas.

O relatório também revelou um defeito na instrumentação: depois da rotação, o exportador lia primeiro o arquivo anterior e podia omitir completamente o arquivo corrente. Por isso, o último `thumbnail_started` visível não pode ser tratado como prova definitiva do ponto exato do crash.

## Correções de estabilidade

- miniaturas de JPG, PNG, TIFF, GIF, WebP e BMP são decodificadas pelo FFmpeg em processo isolado;
- cada processo tem timeout de 45 segundos e é encerrado em caso de bloqueio;
- falha ou abort do decoder não derruba o processo principal do Lumina;
- a validação inicial continua rápida, mas agora limita dimensões a 32.768 pixels e alocação decodificada a 256 MiB;
- o worker continua após arquivos inválidos e registra a falha por item.

## Diagnóstico pós-crash

- o arquivo corrente é exportado antes do histórico rotacionado;
- o limite do relatório foi ampliado de 2.500 para 5.000 eventos;
- a operação de miniatura em andamento fica em marcador persistente;
- após encerramento anormal, a próxima sessão registra a operação que não concluiu;
- caminhos, nomes e hashes continuam removidos do relatório exportado.

## Validação

- suíte Rust completa, incluindo catálogo com 100 mil itens e importação com 2 mil arquivos;
- carga concorrente de 40 miniaturas PNG com consultas da galeria;
- arquivo falso não interrompe o processamento e é classificado como corrompido;
- timeout e encerramento de processo filho cobertos por teste;
- orientação EXIF, RAW, HEIC, vídeo, preview HD e auditoria de cache preservados.

Os diagnósticos fornecidos pelo usuário permanecem locais e fora do Git.
