# Validação — Lumina 0.20.0-beta.1

## Gates automatizados

| Gate | Comando | Critério |
|---|---|---|
| Frontend | `npm.cmd test -- --run` | todos passam |
| Build web | `npm.cmd run build` | TypeScript e Vite sem erro |
| Backend | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1` | todos passam; fixtures opcionais podem ficar ignoradas |
| Formato | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | nenhuma diferença |
| Lints | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | nenhum warning |
| Release | `cargo tauri build --target-dir src-tauri/target-0.20-beta` | EXE, NSIS e MSI gerados |
| Portátil | `scripts/package-0.20-beta.ps1` | manifesto SHA-256 e ZIP gerados |
| Smoke | `scripts/smoke-portable-0.20-beta.ps1` | abre, responde, fecha e remove marcador de sessão |

## Evidências da entrega

- Frontend: 35 testes aprovados em 6 arquivos; TypeScript/Vite aprovado.
- Backend: 116 testes aprovados, zero falhas e 2 fixtures opcionais ignoradas.
- Testes específicos cobrem migração v16, resolução local, idempotência, precedência de nome manual, busca por lugar e interface de correção.
- `cargo fmt --check` e Clippy com `-D warnings` aprovados.
- Release Windows gerado em EXE, MSI e NSIS.
- Smoke portátil: 569 entradas validadas, frontend responsivo, encerramento limpo e working set inicial de 30.973.952 bytes.
- `npm audit` local não alcançou a API por falha de validação do certificado da registry; o mesmo gate permanece obrigatório no CI Windows.

## Artefatos

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| `Lumina-0.20.0-beta.1-portable-windows-x64.zip` | 91.754.013 | `07f8af05f3f6380adc046bf6a252e1e75b9d961591cb9d801435a222ac40c573` |
| `Lumina_0.20.0-1_x64_en-US.msi` | 91.718.356 | `c8d618c757d133bac88e31c4a60aaec64a80f348cc9e730dfc173413a152dd7d` |
| `Lumina_0.20.0-1_x64-setup.exe` | 66.689.066 | `8b5292cb001a960b11840499382685c4742bfcc0a24f2d5b623f29d86d196a30` |
