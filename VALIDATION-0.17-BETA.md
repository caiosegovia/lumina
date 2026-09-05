# Evidências de validação — Lumina 0.17.0-beta.1

Data: 5 de setembro de 2026. Ambiente: Windows x64.

## Gates executados

- Frontend: 30 testes aprovados em 5 arquivos.
- Backend: 103 testes aprovados, 0 falhas e 2 testes de ambiente/escala ignorados pelo gate comum.
- Build TypeScript/Vite: aprovado.
- Rustfmt: aprovado.
- Clippy, todos os targets com `-D warnings`: aprovado.
- npm audit: 0 vulnerabilidades.

## Regressões específicas

- Metadados técnicos enfileirados iniciam automaticamente ao liberar o escritor da biblioteca.
- Pausa deliberada não é apresentada nem consultada como execução ativa.
- Manutenção `lumina://` é excluída da lista de trabalhos humanos.
- Estado de previews e metadados é derivado da fila durável.
- Índice perceptual usa arquivos reais controlados e preserva os originais.
- Descoberta abre a comparação com os dois ativos previamente selecionados.

## Smoke empacotado

- Manifesto: 567 entradas verificadas individualmente por SHA-256.
- Frontend pronto e responsivo em perfil `LOCALAPPDATA` isolado.
- Encerramento limpo e marcador de sessão removido.
- Working set observado: 33.034.240 bytes.

## Artefatos

- Portátil: `bc591f8847897c0b35ae55715d88746874f1e893467000547b448017b0d4d975`
- MSI: `476d47d090c2a5b01592537cdada5268edfa7cd057eeb8d619dc64e0e83b4b14`
- Instalador EXE: `7cabf63d3238a9ce2dee3bc4b16cb02147ffa5c07f5669944b5c528e5bcbab7a`
