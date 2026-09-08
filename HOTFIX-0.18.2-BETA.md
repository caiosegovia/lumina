# Lumina 0.18.2-beta.1

Correção baseada no diagnóstico real exportado em 8 de setembro de 2026 após a 0.18.1.

## Diagnóstico corrigido

O RAW imediatamente anterior ao encerramento terminou com sucesso e voltou a ser processado depois do reinício. Isso descartou a hipótese de um arquivo RAW específico. A queda coincidiu com a geração de preview HD durante navegação e não registrou panic, falta de memória ou perda do heartbeat da interface.

A causa arquitetural encontrada foi a ausência de backpressure antes de `spawn_blocking`: cada mudança de seleção podia criar uma nova tarefa bloqueante, enquanto o limitador real era adquirido somente dentro dela. Respostas assíncronas antigas também podiam atualizar a seleção nova.

## Correções

- apenas uma geração de preview HD pode entrar no pool bloqueante por vez;
- pedidos concorrentes recebem `PREVIEW_BUSY` e o item ainda selecionado tenta novamente;
- navegação aplica debounce de 150 ms;
- resultados de preview, URL e metadados de seleções antigas são descartados;
- extrações RAW de fallback usam os mesmos limites de dimensão e alocação das miniaturas;
- início e conclusão do preview HD ficam registrados no diagnóstico.

## Validação de regressão

- rajada de dez navegações resulta em uma solicitação HD final;
- resposta HD fora de ordem não substitui a foto atual;
- 10.000 tentativas concorrentes não criam fila de threads bloqueantes;
- suítes completas frontend e backend, build, formatação e Clippy.

Os diagnósticos do usuário permanecem fora do versionamento e dos pacotes.
