# Validação — Lumina 0.18.0-beta.1

Documento preenchido durante o gate de release.

## Gates automatizados

- Frontend: **31/31 aprovados em 6 arquivos**
- Backend: **108 aprovados, 0 falhas, 2 ignorados (110 total)**
- TypeScript/Vite: **aprovado**
- Rustfmt: **aprovado**
- Clippy `-D warnings`: **aprovado**
- `npm audit`: **0 vulnerabilidades**
- Pacote Tauri NSIS/MSI: **aprovado**
- Portátil e manifesto: **567 entradas verificadas**
- Smoke do portátil: **frontend pronto, responsivo e encerramento limpo; working set inicial 30.994.432 bytes**

## Artefatos e SHA-256

- Portátil: `3d1ce9edec74cabd844db9f63eb085d6726484cd2a89bb8e4b825d9dab081a17`
- Instalador NSIS: `d9376d2d5a1004d494c9127e50d43c9c81effb4f4e3a87f3edf20039c1439415`
- MSI: `e532bb2d5894e984d0e72421ce52a9a2df19d202fae348398e0ad84c4e88f76a`

O linker MSVC informou apenas a criação normal da biblioteca de importação e do arquivo `.exp`; não houve warning de código nem falha de empacotamento.

## Riscos residuais a homologar

- O teste automatizado comprova limite de lote para 9.300 entradas; o comportamento com as mídias reais e codecs do dispositivo oficial depende do roteiro funcional.
- Pessoas nesta beta são associações nominais deliberadas. Não há reconhecimento facial automático sem modelo local auditado.
- Regiões são coordenadas aproximadas, não nomes de cidades obtidos por serviço externo.
