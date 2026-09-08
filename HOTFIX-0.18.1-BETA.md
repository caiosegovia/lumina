# Lumina 0.18.1-beta.1

Hotfix preparado a partir dos diagnósticos reais exportados em 8 de setembro de 2026.

## Evidências do incidente

- duas sessões terminaram de forma anormal durante a geração intensiva de miniaturas RAW;
- o working set permaneceu entre aproximadamente 20 e 38 MiB;
- o heartbeat da interface permaneceu abaixo de 5 segundos;
- não houve panic Rust, erro de renderização ou encerramento limpo registrado;
- o último trabalho observável antes das quedas foi a extração e decodificação de previews RAW pelo ExifTool.

## Contenção aplicada

- limite de 64 MiB para previews RAW embarcados;
- limite de 32.768 pixels por eixo e 256 MiB de alocação no decodificador;
- panic de uma miniatura isolado como falha do item, sem abandonar o worker;
- log `thumbnail_started` com ID sanitizado e extensão antes da decodificação.

Arquivos de diagnóstico do usuário não fazem parte do commit nem do pacote.
