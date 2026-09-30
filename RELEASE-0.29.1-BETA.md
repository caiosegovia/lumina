# Lumina 0.29.1-beta.1

Correção da regressão de busca da 0.29.0-beta.1, revisão responsiva das seções e melhorias de Atividades.

- Busca com altura estável enquanto carrega; botão e Enter disponíveis, inclusive para repetir uma consulta.
- Resultados obsoletos descartados, paginação coerente, contador preservado e campo sincronizado com visões salvas.
- Busca isolada em componente, grade medida pelo painel e lista adaptável ao inspetor.
- Ordenação, seleção, filtros e pills acessíveis; correções responsivas em Fontes, diálogos, setup e painéis.
- Atividades separa Aguardando de Em execução, permite atualização manual e explica avisos de progresso sem afirmar travamento.
- Histórico da versão reprovada preservado; nenhuma migração de catálogo.

Instaladores: `Lumina_0.29.1-1_x64-setup.exe` e `Lumina_0.29.1-1_x64_en-US.msi`.

Validação local: 64 testes frontend, 150 testes Rust aprovados (2 ignorados), Clippy, build e matriz de 10 cenários de frontend. Smoke do executável extraído do MSI aprovado, incluindo importação, réplica verificada e 50 buscas. Não houve migração do catálogo; a main permanece preservada para homologação.

Consulte [roteiro de homologação](https://github.com/caiosegovia/lumina/blob/v0.29.1-beta.1/VALIDATION-0.29.1-BETA.md) e [revisão do frontend](https://github.com/caiosegovia/lumina/blob/v0.29.1-beta.1/FRONTEND-REVIEW-0.29.1.md).
