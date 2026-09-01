# Evidências de validação — Lumina 0.14.0-beta.2

Data: 2026-09-01. Ambiente: Windows x64, biblioteca de homologação `D:\Galeria Caio`.

## Gates automatizados

- Rust: 97 testes descobertos; 95 aprovados, 0 falhas e 2 ignorados deliberadamente no gate comum.
- Migração: snapshot v13→v14 aprovado e rollback transacional v13 aprovado.
- Resiliência: restart de importação, restart de sincronização, volume offline, espaço insuficiente e ferramenta ausente aprovados.
- Segurança: fontes permanecem inalteradas, promoção exige hash, réplica exige verificação e duplicata sem proteção é bloqueada.
- Frontend: 15/15 testes aprovados, incluindo troca conjunta do arquivo real e metadados ao navegar.
- TypeScript/Vite: build aprovado.
- Clippy: `--all-targets -- -D warnings` aprovado.
- Dependências npm: 0 vulnerabilidades no nível moderado ou superior.

## Escala

- 100.000 itens: dashboard p50 3 ms, p95 4 ms; carga completa 613 ms.
- 500.000 itens: dashboard p50 2 ms, p95 4 ms; carga completa 3.061 ms.
- Leituras concorrentes permaneceram disponíveis durante ambos os cenários.

## Aplicativo empacotado

- Smoke portátil: 563 entradas do manifesto verificadas, frontend pronto e processo responsivo.
- Homologação real: beta.2 iniciou e migrou o catálogo com sucesso; processo `Lumina Ready` responsivo.
- Snapshot externo antes/depois: 1.563 arquivos e 84.315.833.947 bytes, sem alteração fora de `.lumina`.

## Artefatos finais

| Artefato | Bytes | SHA-256 |
|---|---:|---|
| `lumina.exe` | 14.966.784 | `0263842437f9b01ee890c9354a8ee1142217e9de7d620cf602ec7ee4f55d64c7` |
| `Lumina_0.14.0-2_x64_en-US.msi` | 91.677.396 | `99ef66f4a0d70cf750764b67828a9386b616356ca52b3339b865acad7c3dd918` |
| `Lumina_0.14.0-2_x64-setup.exe` | 66.656.534 | `7c6fe56251d476e0cc332d39c7ed557439a2eaa4ca19ebffe2f8d86b527d546b` |
| portátil ZIP | 91.708.186 | `c70c03daed7dbef40fcdb126e10f258d57491c46c2987ec97fe64d093e9ab4c9` |
