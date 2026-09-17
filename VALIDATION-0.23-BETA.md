# Validação 0.23 Beta

## Gates automatizados

| Gate | Resultado |
|---|---|
| Frontend/Vitest | 42 aprovados em 7 arquivos |
| Rust completo | 136 aprovados, 0 falhas, 2 fixtures opcionais ignoradas |
| TypeScript/Vite | aprovado durante o desenvolvimento |
| Formatação e Clippy estrito | aprovados |
| Build release | MSI e NSIS aprovados |
| Smoke portátil isolado | 574 entradas, frontend pronto, encerramento limpo, 31.420.416 bytes de working set |
| Homologação humana | executar `RELEASE-0.23-BETA.md` |

## Cobertura adicionada

- filtro operacional da Atividade entre execução, atenção e histórico;
- tipografia inteiramente local, sem requisição externa;
- agregadores contáveis de duplicatas;
- semântica de confiança e ação nos insights.

Os testes de resiliência, galeria, preview, vídeo, jobs, duplicatas, localização, bursts e insights das versões anteriores permanecem obrigatórios.

## Artefatos

| Pacote | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.23.0-2_x64_en-US.msi` | 92.107.476 | `3e45544d876c1d7c834dc2240005f3db62296910cf178b411cbca345f5c47077` |
| `Lumina_0.23.0-2_x64-setup.exe` | 67.045.342 | `2a51d388357929afbffb4204e195fca8762c0c76077c1a4c32ccb3198d6dbecd` |
| `Lumina-0.23.0-beta.2-portable-windows-x64.zip` | 92.152.643 | `d6cf6dfc57ea3d9013936acba9168347f62931637e809beb083540096e1462d9` |
