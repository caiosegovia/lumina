# Lumina 0.18.4-beta.1

Correção de regressão baseada no diagnóstico real da 0.18.3 exportado em 13 de setembro de 2026.

## Causa observada

A 0.18.3 isolou os decodificadores em processos FFmpeg, mas duas mídias rejeitadas pelo encoder JPEG eram reenfileiradas pela galeria repetidamente. Arquivos RAW sem prévia embarcada também voltavam a ser tentados após reinício. O resultado era uma sequência contínua de processos externos disputando recursos com preview HD, metadados e verificação.

Não houve crescimento de memória na sessão da 0.18.3: o working set observado permaneceu entre aproximadamente 31 e 37 MiB. O problema foi contenção e ausência de um estado terminal efetivo para falhas permanentes.

## Correções

- falhas permanentes de miniatura não voltam para `pending` ao serem solicitadas pela galeria;
- reiniciar o aplicativo preserva o estado `failed` e a contagem de tentativas;
- itens já prontos ou permanentemente falhos não iniciam um novo worker;
- uma nova versão do gerador ou o reparo explícito continuam podendo tentar novamente;
- FFmpeg força `yuvj420p` e compatibilidade `unofficial` ao produzir JPEG, evitando a rejeição de imagens non-full-range no FFmpeg 8.1;
- o isolamento e os timeouts introduzidos na 0.18.3 permanecem ativos.

## Regressões automatizadas

- 100 solicitações consecutivas da galeria mantêm uma única entrada falha, sem aumentar tentativas;
- retomada após reinício não reexecuta falha permanente;
- orientação EXIF, vídeo, RAW, HEIC e geração concorrente permanecem cobertos;
- suíte completa: 112 testes aprovados, zero falhas e dois benchmarks ignorados.

Os diagnósticos do usuário permanecem fora do versionamento.
