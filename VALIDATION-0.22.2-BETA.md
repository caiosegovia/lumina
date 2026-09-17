# Validação 0.22.2 Beta

## Estado

Documento preenchido pelos gates da candidata. A aprovação automatizada não substitui o endurance no dispositivo de produção descrito em `RELEASE-0.22.2-BETA.md`.

## Feedback aberto da homologação

- Tipografia visualmente inconsistente após a 0.22.2: fontes, pesos e hierarquia deixaram partes da interface desorganizadas. Correção planejada para o próximo pacote como revisão do sistema tipográfico completo, sem bloquear a continuidade dos demais testes desta beta.

## Gates

| Gate | Resultado |
|---|---|
| Rust completo | 136 aprovados, 0 falhas, 2 fixtures opcionais ignoradas |
| Frontend/Vitest | 41 aprovados em 7 arquivos |
| Formatação | aprovado |
| Clippy com warnings negados | aprovado |
| TypeScript/Vite | aprovado |
| Build release | MSI e NSIS aprovados |
| Smoke portátil | 573 entradas, frontend pronto, encerramento limpo, 31.010.816 bytes de working set |
| Endurance no dispositivo oficial | homologação humana |

## Evidência funcional preservada

Galeria, visualização, vídeo, comparação, metadados, favoritos, tags, duplicatas, proteção, localização, bursts e insights continuam cobertos pela suíte existente. A 0.22.2 adiciona testes de limites de protocolo, cache LRU, cancelamento em filas, saída limitada de processos, lease de jobs e migração do schema 20.

## Critério de promoção

A tag continua beta até o roteiro humano concluir sem crash, sem crescimento progressivo de memória e sem job preso. Os hashes dos pacotes serão registrados aqui após o build limpo.

## Artefatos

| Pacote | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.22.2-1_x64_en-US.msi` | 91.804.372 | `94b3a70233e125aa64c6ea17686580ce883f5defcee374939fd7802526ebc76d` |
| `Lumina_0.22.2-1_x64-setup.exe` | 66.750.706 | `524568053b6646c27bec7db1421911a3ba228468f8c1d829ab2d4bbd949e2a6e` |
| `Lumina-0.22.2-beta.1-portable-windows-x64.zip` | 91.855.087 | `3bdd890545d3529b3611494e18f11af4fac6a90dc1d0573454031691e27289e6` |
