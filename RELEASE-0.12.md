# Lumina 0.12.0 — inventário e inteligência de armazenamento

## Resultado

A 0.12 preserva o carregamento instantâneo por snapshot e adiciona uma atualização profunda assíncrona. A visão geral agora explica capacidade física, espaço administrado pelo Lumina, réplica, reserva, crescimento, composição, cobertura técnica e pendências acionáveis.

O inventário técnico foi ampliado para formatos de foto, RAW e vídeo. Fotos e RAW são consultados pelo ExifTool em lotes de até 200 arquivos; vídeos registram contêiner, codecs, FPS, bitrate e pixel format pelo FFprobe. O processo é persistente, retomável, cancelável, idempotente e de baixa prioridade.

## Segurança

- Nenhuma rotina do inventário escreve nas fontes.
- TIFF e formatos reconhecidos continuam preservados quando o decoder de preview não é suficiente.
- Cache, catálogo e temporários permanecem separados dos originais.
- Os testes de release usam catálogos temporários; o catálogo 0.11 em execução não foi alterado.

## Benchmark de desenvolvimento

Perfil Rust sem otimização, Windows, leitura concorrente ativa:

| Catálogo | Snapshot p50 | Snapshot p95 | Atualização completa |
|---:|---:|---:|---:|
| 100.000 | 2 ms | 3 ms | 817 ms |
| 500.000 | 3 ms | 214 ms | 3.227 ms |

A atualização completa ocorre depois que o snapshot já foi exibido e não bloqueia a navegação. As medidas reais da importação anterior continuam disponíveis na seção de desempenho do aplicativo.

## Validação automatizada

- 82 testes do núcleo aprovados; 2 testes opcionais ignorados por dependerem de fixture RAW local.
- 13 testes do frontend aprovados.
- Benchmark explícito de 100 mil e 500 mil aprovado.
- Build TypeScript/Vite, formatação Rust, auditoria de dependências e whitespace fazem parte do gate `scripts/verify-0.12.ps1`.
- Portátil e instaladores passam por smoke isolado e recebem manifesto SHA-256.

## Roteiro de teste manual

1. Abra o portátil 0.12 e confirme que a visão geral aparece primeiro com o snapshot.
2. Observe a indicação “Atualizando análises em segundo plano” e continue navegando.
3. Confira capacidade e espaço livre do acervo e da réplica, sem confundir uso total do disco com bytes administrados pelo Lumina.
4. Confira “Ritmo do acervo”, composição, formatos, câmeras e codecs; clique em barras/linhas para abrir a galeria filtrada.
5. Inicie “Atualizar inventário técnico”, acompanhe em Atividade, pause, retome e confirme que a galeria continua responsiva.
6. Verifique cobertura de previews, codecs, itens para revisão e extensões divergentes.
7. Importe uma amostra com JPEG, TIFF, HEIC, DNG/CR2 e vídeo; confirme que arquivos sem preview aparecem como preservados, não corrompidos.

## Limites conhecidos

- O Lumina confirma a réplica local, não o upload remoto do Google Drive.
- Alguns formatos de preservação podem não ter preview nesta versão.
- O enriquecimento inicial de um acervo antigo pode levar tempo, mas roda em lotes e pode ser retomado.
