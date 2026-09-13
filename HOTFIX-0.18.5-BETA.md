# Lumina 0.18.5-beta.1

## Evidência do dispositivo de homologação

O diagnóstico `20260913-222301` confirmou que a 0.18.4 eliminou o
reenfileiramento infinito de miniaturas: 9.217 itens estavam prontos, 52 em
falha terminal e nenhum voltou a processar. A sessão ainda terminou
anormalmente durante navegação no inspetor, imediatamente após gerar a prévia
embarcada de um RAW e consultar seus metadados.

## Correções

- previews RAW embarcados passam a ser decodificados e normalizados somente em
  processo FFmpeg descartável; nenhum codec RAW roda dentro do processo do app;
- o protocolo que entrega a imagem ao WebView ficou estritamente somente
  leitura e não pode iniciar decodificação no thread de requisição;
- a versão do cache de preview foi incrementada para impedir reuso de artefatos
  anteriores;
- enriquecimento ExifTool sob demanda ganhou backpressure: navegação rápida
  combina solicitações concorrentes e devolve imediatamente o snapshot do
  catálogo, sem acumular threads bloqueantes;
- logs identificam início, fim e combinação das requisições de metadados por
  asset anonimizado.

## Critério de homologação

Abrir o inspetor e navegar continuamente por fotos e RAWs durante 30 minutos,
inclusive nos arquivos que precederam a queda. A interface deve permanecer
responsiva, o consumo de memória deve estabilizar e o diagnóstico não deve
registrar uma nova sessão anormal.
