# Lumina 0.14.0-beta.2

O pacote MSI usa a versão técnica `0.14.0-2`, equivalente a esta beta, por exigência do formato Windows Installer.

Beta funcional para homologação no dispositivo oficial. Fontes permanecem somente leitura e o plano de duplicatas nunca exclui arquivos.

## Entregue

- Sincronização incremental e retomável de fontes, com ocorrências presentes/ausentes e reuso de evidências.
- Central de Revisão com contagens reais, filtros, ações rápidas, avanço e desfazer última edição.
- Visualizador com troca sincronizada do arquivo real, filmstrip identificado, tela cheia, zoom, pan e reprodução de vídeo por streaming local com suporte a Range.
- Metadados detalhados de captura, arquivo, imagem, vídeo, localização, origem e proteção.
- Duplicatas com comparação lado a lado, decisões persistentes por grupo e ocorrência, elegibilidade por proteção, simulação e relatório JSON.
- CRUD de visões salvas, álbuns inteligentes, álbuns manuais e tags, além de organização em lote.
- Saúde operacional de catálogo, discos, fontes, miniaturas, proteção, trabalhos, ExifTool, FFmpeg e FFprobe.

## Segurança e limites

- Nenhuma exclusão física está habilitada nesta beta.
- O relatório de limpeza é uma simulação e pode conter caminhos locais para permitir auditoria pelo proprietário.
- Similaridade visual, reconhecimento facial, nuvem e acesso remoto permanecem fora do ciclo.
- Use a galeria de homologação e mantenha as cópias originais independentes durante os testes.

## Roteiro de aceite

1. Atualize uma fonte e confirme progresso e resultado após reiniciar o aplicativo.
2. Percorra as filas da Central de Revisão, edite um item e teste **Desfazer última alteração**.
3. Navegue pelo preview com setas, filmstrip e tela cheia; reproduza um vídeo.
4. Gere um plano de duplicatas e exporte o relatório; confirme que nada foi removido.
5. Crie uma visão inteligente, renomeie/exclua tags e confirme persistência após reabrir.
6. Confira a tela de Proteção e os diagnósticos de saúde.
