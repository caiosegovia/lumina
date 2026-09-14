# Validação — Lumina 0.19.0-beta.1

## Gate automatizado

| Gate | Comando | Critério |
|---|---|---|
| Frontend | `npm.cmd test -- --run` | todos os testes passam |
| Build web | `npm.cmd run build` | TypeScript e Vite sem erro |
| Backend | `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1` | todos passam; fixtures opcionais podem ficar ignoradas |
| Formato | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | nenhuma diferença |
| Lints | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | nenhum warning |
| Release | `cargo tauri build` | EXE, NSIS e MSI gerados |
| Portátil | `scripts/package-0.19-beta.ps1` | manifesto SHA-256 e ZIP gerados |
| Smoke | `scripts/smoke-portable-0.19-beta.ps1` | abre, responde, fecha e remove marcador de sessão |

## Evidência do desenvolvimento

- Frontend: 34 testes aprovados após Atividade, Duplicatas e Proteção.
- Backend: 114 aprovados, zero falhas e 2 fixtures opcionais ignoradas; teste transacional final repetido isoladamente.
- TypeScript/Vite, `cargo fmt --check` e Clippy com `-D warnings` aprovados.
- Release Windows gerado em EXE, MSI e NSIS.
- Smoke portátil: 568 entradas validadas pelo manifesto, frontend pronto, processo responsivo, encerramento limpo, marcador de sessão removido e working set inicial de 32.796.672 bytes.
- CI Windows serializa a suíte Rust para não disputar CPU entre testes de estresse e asserções de timeout de processos externos.

## Artefatos

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| `Lumina-0.19.0-beta.1-portable-windows-x64.zip` | 91.740.378 | `3c612b1c5bfada9c79224a0d779d6c0b3bb33226ad32570276943715930563f3` |
| `Lumina_0.19.0-1_x64_en-US.msi` | 91.714.260 | `0300e6be93cbb5a57d33f84c7946d17cf51c837103bffd7eccbd83e9fcc7e7e0` |
| `Lumina_0.19.0-1_x64-setup.exe` | 66.671.474 | `afb1b22d15d8eb3837b05bbfde291920d0db2a96939dd6aff5e3b85d5699a80e` |

A homologação manual segue `RELEASE-0.19-BETA.md` e é necessária antes de promover a beta.
