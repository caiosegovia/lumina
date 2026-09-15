# Validação — Lumina 0.21.0-beta.1

| Gate | Comando | Critério |
|---|---|---|
| Frontend | `npm.cmd test -- --run` | todos passam |
| Build web | `npm.cmd run build` | TypeScript/Vite sem erro |
| Backend | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1` | todos passam; fixtures opcionais podem ficar ignoradas |
| Formato | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | sem diferença |
| Lints | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | sem warning |
| Release | build Tauri em `src-tauri/target-0.21-beta` | EXE, MSI e NSIS |
| Portátil | `scripts/package-0.21-beta.ps1` | ZIP e manifesto SHA-256 |
| Smoke | `scripts/smoke-portable-0.21-beta.ps1` | abre, responde e fecha limpo |

## Cobertura nova

- Hora EXIF local, offset presente, data sem horário e migração v17.
- Coerência temporal, técnica e visual de bursts.
- Curadoria reversível da recomendação.
- Resolução offline, override idempotente e filtro geográfico exato.

## Evidências executadas em 15/09/2026

- Frontend: 38 testes aprovados em 6 arquivos.
- Backend: 120 testes aprovados, 0 falhas e 2 fixtures opcionais ignoradas.
- Build web: aprovado.
- `cargo fmt --check`: aprovado.
- `cargo clippy --all-targets -- -D warnings`: aprovado, sem warnings.
- Build Tauri release: aprovado; MSI e NSIS gerados.
- Smoke portátil: 570 entradas verificadas, frontend carregado, encerramento limpo e working set de 30.736.384 bytes.

## Artefatos reproduzíveis

| Artefato | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.21.0-1_x64-setup.exe` | 66.700.529 | `35688ca86f908429bcf981a71f6563437ad9a5f177384ef88aef101d77a4935e` |
| `Lumina_0.21.0-1_x64_en-US.msi` | 91.734.740 | `7eb6371a9f553c4f2e1537b67a38a154c58d71e506cf3c8845d8b33deedd67eb` |
| `Lumina-0.21.0-beta.1-portable-windows-x64.zip` | 91.765.712 | `de25c8f80b46297117184d97516d3a73bb8635dc61543560850efc7c458560f9` |

O manifesto interno do pacote portátil permite verificar individualmente o executável, o frontend, os recursos e as ferramentas embarcadas.
