# Validação — Lumina 0.22.0-beta.1

| Gate | Critério |
|---|---|
| Frontend | todos os testes Vitest passam |
| Web | TypeScript e Vite sem erro |
| Backend | todos os testes Rust passam; fixtures opcionais podem ser ignoradas |
| Qualidade | `cargo fmt --check` e Clippy sem warnings |
| Migração | v18 transacional, idempotente e preservando overrides |
| Release | EXE, MSI, NSIS e ZIP portátil gerados |
| Smoke | frontend pronto e encerramento limpo em perfil isolado |

## Evidências executadas em 15/09/2026

- Frontend: 39 testes aprovados em 6 arquivos.
- Backend: 123 testes aprovados, 0 falhas e 2 fixtures opcionais ignoradas.
- Build web, `cargo fmt --check` e Clippy com `-D warnings`: aprovados.
- Prova ExifTool: arquivo sintetizado com cidade, estado, país e ponto de interesse confirmou precedência dos campos embutidos sobre resultado geográfico inferior.
- Build Tauri release: MSI e NSIS gerados.
- Smoke portátil final: 571 entradas verificadas, frontend pronto, encerramento e limpeza completos, working set de 30.736.384 bytes.

| Artefato | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.22.0-1_x64-setup.exe` | 66.707.100 | `59c207ef6344a71e3168e7c599967aeb6708de61a24765881ac245dce1abb63c` |
| `Lumina_0.22.0-1_x64_en-US.msi` | 91.738.836 | `46c426eaae1c2797886a05cb14db078a4c8a5bf201404b94d3f386cdfbbbaade` |
| `Lumina-0.22.0-beta.1-portable-windows-x64.zip` | 91.784.229 | `7b789f0bbce53d622d51778404d2ee548c3cf3baa296f84b2de5ce5137bd5f3a` |
