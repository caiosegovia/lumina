# Validação 0.23 Beta

## Gates automatizados

| Gate | Resultado |
|---|---|
| Frontend/Vitest | 42 aprovados em 7 arquivos |
| Rust completo | 136 aprovados, 0 falhas, 2 fixtures opcionais ignoradas |
| TypeScript/Vite | aprovado durante o desenvolvimento |
| Formatação e Clippy estrito | aprovados |
| Build release | MSI e NSIS aprovados |
| Smoke portátil isolado | 574 entradas, frontend pronto, encerramento limpo, 28.348.416 bytes de working set |
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
| `Lumina_0.23.0-1_x64_en-US.msi` | 91.804.372 | `9d59a694062f3646614c9612398be31d8d64e10461950e0a479a559d935cc985` |
| `Lumina_0.23.0-1_x64-setup.exe` | 66.732.785 | `c9b58a28df37a2e4d91a1fd046fd4a84f956b593513480e33ce9d4fd5ebd32da` |
| `Lumina-0.23.0-beta.1-portable-windows-x64.zip` | 91.857.042 | `81219a5713abec2c24e1ba935ec7522a98120c4f6fb777180b7ff0cfcc43b324` |
