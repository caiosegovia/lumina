# Validação 0.23 Beta

## Gates automatizados

| Gate | Resultado |
|---|---|
| Frontend/Vitest | 43 aprovados em 7 arquivos, incluindo persistência dos três modos de tema |
| Rust completo | 136 aprovados, 0 falhas, 2 fixtures opcionais ignoradas |
| TypeScript/Vite | aprovado durante o desenvolvimento |
| Formatação e Clippy estrito | aprovados |
| Build release | MSI e NSIS aprovados |
| Smoke portátil isolado | 575 entradas, frontend pronto, encerramento limpo, 31.166.464 bytes de working set |
| Homologação humana | executar `RELEASE-0.23-BETA.md` |

## Cobertura adicionada

- temas Claro, Escuro e Seguir Windows com preferência persistente;
- DM Sans e Manrope embarcadas, sem rede ou fallback inesperado;
- tokens semânticos e revisão das superfícies principais nos dois temas;
- filtro operacional da Atividade entre execução, atenção e histórico;
- tipografia inteiramente local, sem requisição externa;
- agregadores contáveis de duplicatas;
- semântica de confiança e ação nos insights.

Os testes de resiliência, galeria, preview, vídeo, jobs, duplicatas, localização, bursts e insights das versões anteriores permanecem obrigatórios.

## Artefatos

| Pacote | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.23.0-3_x64_en-US.msi` | 91.923.156 | `a6d121758a4c3154f2be4a88a37d31e5fe67312872193029bfe87f68e39f0b9e` |
| `Lumina_0.23.0-3_x64-setup.exe` | 66.876.352 | `8af2a2e02e43f71ec14596f4429aa20fcc4e3b1d8d587c7f590afbc2ffe35908` |
| `Lumina-0.23.0-beta.3-portable-windows-x64.zip` | 91.992.003 | `375bda0f1eb77752a6b41d822d68d8b348252989ee05bc818f1c957737a8e10c` |
