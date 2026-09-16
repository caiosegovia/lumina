# Validação 0.22.1 Beta

## Resultado

- Frontend: 40 testes aprovados em 6 arquivos, incluindo 500 trocas rápidas e garantia de que fotos não pedem a rota original.
- Rust: 126 aprovados, 0 falhas e 2 fixtures opcionais ignoradas.
- Motor de insights: recorte, amostragem, análise completa, cache e schema 19 aprovados.
- Localização: precedência nativa v3.1 e célula precisa aprovadas.
- `cargo fmt --check`, Clippy `-D warnings` e TypeScript/Vite aprovados.
- Build release limpo concluído; mensagem do linker sobre criação de `.lib/.exp` é informativa.
- Smoke portátil: 571 entradas verificadas, frontend pronto, encerramento limpo, working set inicial 30.760.960 bytes.

## Artefatos

| Pacote | Bytes | SHA-256 |
|---|---:|---|
| `Lumina_0.22.1-1_x64_en-US.msi` | 91.792.084 | `c0ab4c36078e7e51dbbaefd009a10f6f667661bfe992c9f5d587f1e60594d8a7` |
| `Lumina_0.22.1-1_x64-setup.exe` | 66.728.756 | `d488d6afbd897251db2f808896c6950a3d91e5cc291b1596e44d50dec1ebe732` |
| `Lumina-0.22.1-beta.1-portable-windows-x64.zip` | 91.827.679 | `850d058e6448707e89866cde413c95c7d7e0f1bc4f37ddb16d3adb752f7ce16a` |

## Homologação humana restante

Executar o roteiro `RELEASE-0.22.1-BETA.md` no dispositivo de produção com a galeria real. O teste automatizado prova limites e contratos, mas não substitui a observação prolongada do WebView2 e dos drivers daquele equipamento.
