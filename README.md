# Lumina

Versão em desenvolvimento: **0.17.0-beta.1**, ciclo de descoberta e curadoria local. A **0.16.0-beta.1** permanece publicada e congelada como baseline histórica. Consulte [SPEC-0.17-BETA.md](SPEC-0.17-BETA.md), [ARCHITECTURE-0.17.md](ARCHITECTURE-0.17.md) e [CLOSURE-0.16.md](CLOSURE-0.16.md).

A anatomia, os wireframes responsivos e os estados da nova galeria estão registrados em [DESIGN-0.15-GALLERY.md](DESIGN-0.15-GALLERY.md).

Lumina é um aplicativo desktop local para inventariar, consolidar, deduplicar e proteger grandes acervos de fotos e vídeos espalhados por discos, cartões e pastas.

> Projeto em desenvolvimento ativo. Antes de usar com um acervo importante, mantenha cópias independentes e valide o fluxo com uma amostra.

## Princípios

- As fontes são somente leitura: o Lumina não move, edita nem exclui os arquivos originais.
- A cópia é promovida ao acervo somente depois da verificação SHA-256.
- Conteúdo idêntico ocupa um único arquivo no acervo, preservando as ocorrências de origem.
- Catálogo, cache, relatórios e temporários ficam separados dos originais.
- “Protegido” confirma a réplica local; não afirma que o Google Drive terminou o upload remoto.

## Funcionalidades atuais

- Análise recursiva, fontes persistentes/offline e exclusão de diretórios de sistema.
- Deduplicação exata, hash seletivo e hash integrado à cópia.
- Inventário amplo de fotos, vídeos e RAW com ExifTool em lotes, FFmpeg e FFprobe empacotados.
- Importação retomável, controles de trabalhos e relatórios técnicos.
- Galeria em grade ou lista, paginação por cursor, filtros rápidos, pills, preview HD progressivo e metadados EXIF sob demanda.
- Dashboard progressivo com capacidade dos discos, composição, crescimento, saúde técnica, proteção e insights acionáveis.
- Catálogo SQLite com snapshots, agregados e índices testados com 100 mil e 500 mil mídias.

## Stack

- Tauri 2 e Rust
- React e TypeScript
- SQLite
- ExifTool, FFmpeg e FFprobe

## Desenvolvimento no Windows

Pré-requisitos: Node.js 22, Rust estável, Microsoft C++ Build Tools, WebView2 Runtime e Git LFS.

```powershell
git lfs pull
npm.cmd ci
npm.cmd run tauri dev
```

O modo navegador usa dados demonstrativos e não toca no sistema de arquivos:

```powershell
npm.cmd run dev
```

## Validação

```powershell
npm.cmd test
npm.cmd run build
cargo test --manifest-path src-tauri/Cargo.toml
```

A versão beta atual possui gates automatizados do núcleo e frontend, além dos benchmarks de release. Consulte [RELEASE-0.14-BETA.md](RELEASE-0.14-BETA.md).

## Build

```powershell
npm.cmd run tauri build
```

Os artefatos locais são gerados abaixo de `src-tauri/target/release/bundle/` e não fazem parte do repositório.

## Privacidade e segurança

O processamento é local. Antes de abrir uma contribuição ou issue, remova caminhos pessoais, nomes de arquivos, coordenadas, metadados e trechos do catálogo. Consulte [SECURITY.md](SECURITY.md).

## Licença

Ainda não há uma licença de código aberto definida. Até que uma seja adicionada, todos os direitos sobre o código são reservados ao autor. Dependências e ferramentas de terceiros mantêm suas próprias licenças; veja [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
