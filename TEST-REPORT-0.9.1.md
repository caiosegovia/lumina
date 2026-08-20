# Relatório de validação — Lumina 0.9.1

- Testes Rust: 49 aprovados, 0 falhas.
- Testes React: 10 aprovados, 0 falhas.
- Build web de produção: aprovado.
- RAW: fixture real validada pelo teste do núcleo.
- Falha `perl532.dll code 126`: classificada como dependência indisponível por teste dedicado.
- Empacotamento: exige ao menos 500 arquivos auxiliares do ExifTool e execução bem-sucedida de `exiftool -ver`.
- Build desktop release: aprovado.
- Pacote: 566 entradas no manifesto e 552 arquivos auxiliares do ExifTool.
- ExifTool extraído do ZIP: versão 13.59, inicialização aprovada.
- DNG validado com o ExifTool do diretório portátil: `Validate: OK`, código 0.
- Smoke Windows: frontend pronto e janela nativa responsiva em perfil limpo.
- SHA-256 do ZIP: `d8bfb173d7a9bb4546c691fc235fc69b294051a25122d552d0150c9d03327229`.
