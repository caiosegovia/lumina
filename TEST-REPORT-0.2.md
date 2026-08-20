# Relatório de validação — Lumina 0.2.0

Data: 17 de agosto de 2026  
Ambiente: Windows x64, Rust MSVC, Node/Vite e WebView2

## Resultado automatizado

- Formatação Rust, 35 testes Rust, 6 testes frontend e build web: aprovados pelo gate estrito.
- Fixtures reais aprovadas: JPG, HEIC, DNG RAW e MP4.
- Carga aprovada: 100.000 ativos SQLite, 2.000 arquivos e vídeo de 64 MiB.
- Fontes preservadas por bytes/hash em sucesso, corrupção e cancelamento.
- Timeout, cancelamento, concorrência, saída grande, sanitização e dependência ausente aprovados.

O log definitivo é regenerado imediatamente antes do pacote com `scripts/verify-0.2.ps1`, que para na primeira falha.

## Distribuição

- Executável release e cabeçalho PE `Windows GUI`: aprovados.
- MSI x64 e NSIS x64: gerados.
- Portátil x64: ferramentas/licenças incluídas, com `MANIFEST.json` e SHA-256 externo.

## Aceite funcional coberto

O fluxo automatizado cria biblioteca, analisa em segundo plano, navega, consolida, mostra progresso geral/etapa, pausa, retoma, cancela, recupera, abre galeria, usa miniaturas/placeholder, exporta recibo, tenta novamente e verifica backup.

“Backup verificado” significa cópia local conferida por SHA-256; não confirma o upload remoto do Google Drive.
