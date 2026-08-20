# Estabilização arquitetural

Objetivo do produto: consolidar e tornar pesquisáveis dezenas ou centenas de milhares de mídias espalhadas, sem alterar origens, mantendo integridade, proteção verificável, retomada e navegação fluida enquanto trabalhos longos continuam.

| # | Problema | Plano obrigatório | Evidência de aceite |
|---|---|---|---|
| 1 | Trabalho pesado no caminho da interface | Tornar leitura de miniatura não bloqueante e enfileirar geração | Teste prova retorno imediato sem executar gerador |
| 2 | Inicialização repetida em cada conexão | Separar migração de abertura de conexão | Contador/teste prova uma inicialização por biblioteca |
| 3 | Ausência de coordenador global de recursos | Introduzir limites compartilhados por CPU, disco e processo | Teste de pico de concorrência e prioridade interativa |
| 4 | Fila não representa todo trabalho | Persistir miniatura, análise, hash, validação, cópia, backup e verificação | Testes de criação, retomada e estados da fila |
| 5 | Gerenciador parcialmente volátil | Reconstruir execução e prioridades pelo catálogo | Teste de reinício com itens processing/pending |
| 6 | Conexões SQLite sem política | Serviço de catálogo com leitura configurada e escritor em lotes | Teste de contenção e número limitado de conexões/escritas |
| 7 | N+1 na galeria | Carregar fontes e tags da página em consultas agrupadas | Teste/contador limita consultas por página |
| 8 | Filtros incompatíveis com índices | Normalizar campos consultáveis e conferir query plans | Testes EXPLAIN QUERY PLAN nos filtros principais |
| 9 | Miniaturas em Base64 | Servir arquivos por protocolo local seguro | Teste valida URL/path autorizado e ausência de Base64 |
| 10 | Muitos processos por RAW | Usar extração coordenada e metadado de orientação já catalogado | Teste limita processos externos por miniatura |
| 11 | Fonte percorrida mais de uma vez | Inventariar uma vez e processar a tabela persistida | Teste conta uma enumeração física por análise |
| 12 | Identidade de fonte baseada em letra | Persistir GUID/serial do volume e caminho relativo | Teste diferencia dois volumes na mesma letra |
| 13 | Snapshot SQLite inseguro | Usar backup consistente do SQLite | Teste restaura snapshot durante WAL ativo |
| 14 | Erros de proteção ignorados | Propagar falhas de catálogo e manifesto | Testes de falha impedem estado protegido/concluído |
| 15 | Manifesto append-only frágil | Gerar manifesto versionado por troca atômica | Teste de reconstrução, checksum e arquivo parcial |
| 16 | Verificação integral serial/bloqueante | Tornar verificação persistente, paginada e limitada | Teste de progresso, cancelamento e retomada |
| 17 | Histórico sem retenção | Definir auditoria permanente e compactar telemetria | Teste de retenção preserva eventos essenciais |
| 18 | Modelos duplicados podem divergir | Centralizar transições em transações com invariantes | Testes de consistência após falha injetada |
| 19 | Estados como strings distribuídas | Introduzir enums e conversão centralizada | Teste cobre serialização e todas as transições válidas |
| 20 | Progresso escrito em excesso | Agregar e persistir por intervalo/lote | Benchmark limita escritas por segundo |
| 21 | Teste de escala irreal | Cenário com 100 mil registros e arquivos/miniaturas reais representativos | Relatório com p50/p95, memória e throughput |
| 22 | Smoke mede abertura, não usabilidade | Testar navegação sob processamento concorrente | E2E mede latência e ausência de bloqueio sob carga |

Nenhum item será marcado como concluído apenas por compilação. O checklist final será derivado das evidências executadas nesta branch.
