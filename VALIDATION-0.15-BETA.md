# Evidências de validação — Lumina 0.15.0-beta.1

Data: 2026-09-02. Ambiente: Windows x64.

## Gates aprovados

- Frontend: 19/19 testes aprovados.
- Backend: 98 testes aprovados, 0 falhas e 2 testes opcionais ignorados.
- TypeScript e Vite: build de produção aprovado.
- Clippy: `--all-targets -- -D warnings` aprovado.
- Dependências npm: 0 vulnerabilidades conhecidas.
- Smoke portátil: 567 entradas verificadas, frontend responsivo e encerramento limpo.
- Memória no smoke documental: 28.459.008 bytes.

## Contratos novos cobertos

- A lista apresenta colunas operacionais estáveis.
- O inspetor pertence ao workspace e não cobre a galeria em telas amplas.
- A sequência é explicada, recolhível e tem preferência persistente.
- Grade, lista, virtualização, preview HD, EXIF e ações em lote permanecem cobertos pela regressão.

## Artefatos

| Artefato | SHA-256 |
|---|---|
| `Lumina-0.15.0-beta.1-portable-windows-x64.zip` | `45db7d3b6bdb97153d1478c0eb0dcef9ee07c3a6247ed25365a63c690906ae4a` |
| `Lumina_0.15.0-1_x64_en-US.msi` | `07a034ca46e8ebb743960c79f2095168c87f4e6f5a0161c8b24b9444dc4bd4ad` |
| `Lumina_0.15.0-1_x64-setup.exe` | `9374bbc8279b68504799f92148646b1233b8d55b643d9c9cc6140a5bdb2716b8` |

Esta candidata está tecnicamente liberada para homologação visual e funcional. A promoção para `main` depende do aceite do novo workspace da galeria.
