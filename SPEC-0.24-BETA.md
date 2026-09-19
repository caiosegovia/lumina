# Lumina 0.24 beta — acervo confiável e operação transparente

## Objetivo

Eliminar estados presos e falsos alertas observados em uma biblioteca real de aproximadamente 9.300 arquivos, tornar o processamento em segundo plano auto-recuperável e corrigir integrações do Windows e diagnósticos sem alterar originais.

## Entrega

- Fila durável como única fonte de verdade, com lease e watchdog de redisparo.
- Recuperação de itens `processing` cujo lease expirou.
- Encadeamento automático de proteção pendente, sem exigir reinício do app.
- Estado operacional da interface baseado também em `pending` e `processing`; cópia longa não é rotulada como travamento apenas pela idade do relógio.
- Prévia RAW usa diretamente a imagem embarcada, evitando tentativas FFmpeg sabidamente incompatíveis.
- Explorer usa `SHOpenFolderAndSelectItems`, com seleção nativa do arquivo.
- Diagnóstico schema v4 contém instante UTC, horário local e offset; o nome do arquivo usa a hora local.
- Limite máximo de mídia mantido em 64 GiB e todos os subprocessos permanecem limitados.

## Invariantes

- Originais e fontes externas são somente leitura.
- Nenhum destino existente é sobrescrito.
- Um único writer altera a biblioteca.
- Estados terminais dependem de seus artefatos persistidos.
- Falha de preview não invalida nem remove a mídia.
- Horário de captura sem offset permanece wall-clock; UTC de telemetria nunca é usado como data da foto.

## Fora do escopo

Reconhecimento de pessoas, verificador executável independente, edição de originais, mapas completos e sincronização em nuvem.
