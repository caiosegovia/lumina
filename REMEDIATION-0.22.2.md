# Plano de remediação — Lumina 0.22.2

## Regra de execução

Não há feature nova nesta release. Cada onda começa apenas após os testes da anterior. Um item só muda para concluído quando código, teste, medição e documentação estiverem no mesmo commit ou PR.

## Onda 0 — reprodução e proteção do laboratório

1. Congelar fixture anonimizada que reproduza o pico, incluindo vídeo esparso de 64 GiB e ranges hostis.
2. Criar medidor da árvore de processos e linha de base no equipamento mínimo.
3. Transformar os logs anexados em casos de regressão sem publicar dados pessoais.
4. Adicionar kill switch de trabalho derivado para sessões de diagnóstico.

Saída: causa reproduzível, baseline e teste vermelho para ARC-001/002.

## Onda 1 — fechar alocações sem limite

1. Implementar `BoundedRange` com máximo de 4 MiB, suporte correto a intervalo aberto/sufixo e `416` para inválidos.
2. Nunca responder vídeo inteiro; inclusive requisição sem `Range` recebe primeiro bloco limitado.
3. Limitar stdout/stderr durante a leitura, não depois; drenar para impedir deadlock.
4. Aplicar limite central de 64 GiB no inventário, hash, validação, preview, cópia, proteção e mídia.
5. Converter exportações e insights completos para streaming/agregação limitada.

Saída: nenhum tamanho externo controla `Vec`; testes de 0 B, 4 MiB, 64 GiB, acima de 64 GiB, range múltiplo, invertido e overflow.

## Onda 2 — runtime supervisionado

1. Introduzir `ResourceGovernor` único com permits de memória, I/O, CPU/processo e prioridades.
2. Substituir threads destacadas por tasks/workers supervisionados e registry de handles.
3. Tornar aquisição e execução canceláveis com deadline.
4. Implementar shutdown em fases: parar admissão, cancelar derivados, checkpoint de jobs, encerrar filhos, join e flush.
5. Aplicar degradação em 80%, 90% e limite absoluto.

Saída: testes de concorrência, cancelamento em espera, encerramento sob carga e ausência de workers após fechar.

## Onda 3 — jobs e persistência verdadeiros

1. Criar máquina de estados tipada única e tabela de transições válidas.
2. Scheduler consulta jobs por lease durável; `active` em memória vira apenas cache.
3. Heartbeat, expiração, retry máximo, backoff e recuperação ficam persistidos.
4. Operações de réplica usam transação única; configuração usa escrita atômica.
5. Migração usa SQLite backup API, páginas limitadas e cutover idempotente.
6. Eliminar descarte silencioso de erros críticos.

Saída: fault injection em cada limite transacional e reinício em todas as etapas.

## Onda 4 — frontend e mídia com lifecycle

1. Criar serviço de preview com requisição identificada, coalescência e cancelamento real.
2. LRU limitado para miniaturas/previews e descarte de páginas distantes.
3. Cleanup explícito do vídeo e bloqueio de requisição obsoleta.
4. Unificar eventos de jobs; polling adaptativo apenas como fallback.
5. Remover trabalho pesado automático no mount; snapshot primeiro, refinamento sob orçamento.

Saída: 500/5.000 trocas, scroll longo, fotos/vídeos alternados e navegação durante job dentro dos NFRs.

## Onda 5 — observabilidade, segurança e modularidade

1. Logger estruturado de escritor único e correlação ponta a ponta.
2. Métricas da árvore, WebView, ferramentas, filas, latências p50/p95/p99 e long tasks.
3. Export allowlist e suíte contra vazamento de PII.
4. CSP explícita e hardening dos protocolos/capabilities.
5. Extrair módulos: `application`, `domain`, `ports`, `adapters` e `ui/features`; reduzir god modules progressivamente.

Saída: diagnóstico suficiente para atribuir qualquer pico a operação/componente sem conter dados privados.

## Onda 6 — gates de beta homologável

1. Unitários, integração, property/fuzz de parsers e fault injection.
2. Catálogos de 10 mil e 100 mil; arquivos reais representativos e arquivos esparsos de borda.
3. Soak de 2 h misturando galeria, vídeo, metadados e job.
4. Instalação limpa, upgrade da 0.22.1, rollback do instalador e recuperação pós-kill.
5. Assinar relatório com versões, máquina, commit, médias, p95, p99, picos e hashes dos artefatos.

Saída: `0.22.2-beta` somente se todos os bloqueadores da matriz estiverem verdes.

## Estratégia de branches e commits

- `docs/0.22.2-resilience-audit`: contrato e auditoria; não contém correção funcional.
- `fix/0.22.2-bounded-io`: ondas 0 e 1.
- `fix/0.22.2-supervised-runtime`: onda 2.
- `fix/0.22.2-durable-jobs`: onda 3.
- `fix/0.22.2-ui-lifecycle`: onda 4.
- `fix/0.22.2-observability-hardening`: onda 5.
- `release/0.22.2-beta`: integração e onda 6.

Cada branch deve ser curta, revisável e integrada somente com gate verde. Não reescrever a aplicação de uma vez reduz o impacto nas jornadas já homologadas.
