# Evidências de validação — Lumina 0.14.0-beta.4

Data: 2026-09-02. Ambiente: Windows x64. Biblioteca de homologação: `D:\Galeria Caio`.

## Gates automatizados

- Rust: 100 testes descobertos; 98 aprovados, 0 falhas e 2 fixtures opcionais ignoradas no gate comum.
- Preview HD: imagem de 3000 × 1800 convertida em 2560 × 1536, cache reutilizado e original preservado.
- EXIF sob demanda: câmera, lente, ISO e abertura extraídos por ExifTool, persistidos e relidos.
- Migração: snapshot v13→v14 e rollback transacional continuam aprovados.
- Resiliência: restart, volume offline, espaço insuficiente, ferramenta ausente e filas duráveis aprovados.
- Segurança: fontes somente leitura, promoção por hash, réplica verificada e bloqueio de duplicata sem proteção aprovados.
- Frontend: 16/16 testes aprovados, incluindo troca sincronizada e substituição por preview HD.
- TypeScript/Vite: build aprovado.
- Clippy: `--all-targets -- -D warnings` aprovado.
- Dependências npm: 0 vulnerabilidades no nível moderado ou superior.

## Escala preservada

- 100.000 itens: dashboard p95 abaixo do limite de 300 ms no teste de catálogo.
- 500.000 itens: benchmark de release anterior permanece como referência; consultas usam agregados e índices sem N+1.
- Leituras concorrentes continuam disponíveis durante geração de miniaturas.

## Aplicativo empacotado

- Portátil: 565 entradas verificadas pelo manifesto SHA-256.
- Frontend do ZIP: título `Lumina Ready`, processo responsivo e encerramento limpo.
- Memória inicial no smoke isolado: 31.031.296 bytes.
- Marcador de sessão removido após encerramento normal.
- Snapshot externo antes/depois: 1.563 arquivos e 84.315.833.947 bytes, sem alteração fora de `.lumina`.

## Artefatos beta.4

| Artefato | Bytes | SHA-256 |
|---|---:|---|
| `Lumina.exe` portátil | 15.457.280 | `be3ef4b098333cec8782a4e1d07a197adb14df02ce64b1a03e50843cd9392a51` |
| `Lumina_0.14.0-4_x64_en-US.msi` | 91.865.812 | `3222b9e5be62d8471985871ff6ffbffdca3317761dd7ed0404874fe1efa2c7e9` |
| `Lumina_0.14.0-4_x64-setup.exe` | 66.760.750 | `3c4ab02c0ebc0cc5badb4cf0177e92cc80de59679cbdce406bf93a3fff72da86` |

O hash externo do ZIP é publicado junto da entrega; ele não é incorporado neste documento para evitar uma referência circular dentro do próprio arquivo compactado.

## Homologação humana pendente

Os gates técnicos autorizam a beta.4 para teste no dispositivo oficial. A promoção para versão final depende da validação visual e funcional do responsável pelo produto, especialmente preview HD, metadados reais, Saúde, pills e estados de duplicatas.
