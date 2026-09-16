# Auditoria arquitetural — base 0.22.1

Data de corte: 2026-09-16. Base: commit `9bb6be9`, tag `v0.22.1-beta.1`.

## Conclusão executiva

A base preserva bons fundamentos — originais somente leitura, cópia verificada, SQLite em WAL, paginação da galeria, virtualização, limites de decodificação de imagem e filas persistidas — mas **não atende ainda ao contrato de resiliência da 0.22.2**. O crash homologado não é apenas um bug pontual: o sistema possui vários caminhos sem limite e coordenadores concorrentes que não compartilham orçamento.

A 0.22.1 permanece utilizável apenas em homologação controlada. A evolução funcional está congelada até P0 e P1 serem resolvidos e os gates do documento `TRACEABILITY-0.22.2.md` produzirem evidência.

“Todos os antipatterns” significa aqui uma varredura sistemática da base nesta data. Não é uma promessa de ausência futura: o inventário deve ser atualizado por revisão estática, testes hostis, profiling e análise de cada mudança.

## Cobertura e método

Foram revisados os módulos Rust de catálogo, jobs, engine, mídia, processos, recursos, diagnósticos, galeria, descoberta, duplicatas, armazenamento, sincronização, biblioteca e eventos; os componentes React, API IPC, polling, caches e lifecycle; configuração/capabilities Tauri; workflow de CI; scripts de empacotamento; documentação e os diagnósticos locais anexados. A busca incluiu alocações proporcionais a entrada, coleções integrais, processos/threads, estado global, cancelamento, erros descartados, conexões SQLite, escrita de arquivo, protocolos, timers e caches.

Esta rodada é uma auditoria estática apoiada nos logs. Profiling da árvore, fuzz, fault injection e soak são deliberadamente evidências da implementação futura, não resultados inventados nesta documentação.

| Severidade | Quantidade | Situação |
|---|---:|---|
| P0 | 5 | bloqueiam qualquer beta |
| P1 | 19 | devem fechar antes da homologação |
| P2 | 14 | entram no plano ou recebem risco formal aceito |
| P3 | 2 | governança e qualidade contínua |

## Escala de severidade

- **P0 — bloqueador:** pode encerrar o processo, corromper verdade de negócio ou violar originais.
- **P1 — alto:** pode causar travamento, vazamento progressivo, job preso, perda de retomada ou falha de segurança.
- **P2 — médio:** aumenta acoplamento, custo de mudança, diagnóstico ou risco de regressão.
- **P3 — baixo:** dívida de clareza, consistência ou automação sem impacto operacional imediato.

## Inventário de achados

| ID | Sev. | Antipattern / evidência | Risco | Direção obrigatória |
|---|---|---|---|---|
| ARC-001 | P0 | `lumina-media` usa o arquivo inteiro quando não há `Range` e aceita intervalo explícito sem teto; `lib.rs:2123-2147`. | Alocação proporcional a vídeo de até dezenas de GiB; crash já observado perto de 2,8 GiB. | Parser RFC limitado, resposta máxima de 4 MiB, `416` para intervalo inválido, streaming e testes hostis. |
| ARC-002 | P0 | Runner de processos usa `read_to_end` em stdout/stderr; `process.rs:203-212`. | Ferramenta ou mídia hostil esgota memória antes da validação posterior. | Dreno incremental com buffers circulares limitados; matar árvore no excesso/timeout. |
| ARC-003 | P0 | Não existe política aplicada de 64 GiB; descoberta registra qualquer `metadata.len()`; `engine.rs:643-662`. | Arquivo fora do suporte entra em hash, decode, cópia e protocolo. | Guard central no ingresso e em cada borda; estado `unsupported_oversize`. |
| ARC-004 | P0 | Alteração do destino de réplica executa criação de job, invalidação de assets, fila e configuração sem uma única transação; `lib.rs:149-166`. | Estado parcialmente migrado após erro ou falta de energia. | Use case transacional e configuração atômica após commit verificável. |
| ARC-005 | P0 | Migração coleta todo o acervo, copia o arquivo SQLite após checkpoint e não possui exclusão global de todos os escritores; `library.rs:70-132`. | Snapshot inconsistente e uso de memória proporcional ao catálogo. | API SQLite backup, lease exclusivo durável, paginação e cutover recuperável. |
| ARC-006 | P1 | Limitadores de processo e I/O são independentes; previews, metadados, vídeo e jobs não compartilham memória/CPU; `process.rs:90-120`, `resource.rs:46-68`. | Soma de operações “limitadas” ultrapassa o equipamento mínimo. | `ResourceGovernor` único, prioridades, pesos, pressão e admissão. |
| ARC-007 | P1 | Telemetria mede só o processo Rust; `diagnostics.rs:132-160`. | WebView2 e filhos podem consumir GiB sem alerta nem atribuição. | Medir Job Object/árvore de processos e picos por operação. |
| ARC-008 | P1 | Escrita e rotação de log não são sincronizadas; `diagnostics.rs:58-85`. Logs reais já exibiram linhas intercaladas. | Diagnóstico corrompido e corrida na rotação. | Único writer assíncrono, fila limitada, flush e rotação atômica. |
| ARC-009 | P1 | Monitor é thread infinita sem handle/cancelamento; `diagnostics.rs:207-241`; pode ser iniciado no startup e na criação. | Worker órfão, duplicação e acesso a biblioteca antiga. | Supervisor de serviços com token e `join` no shutdown/troca de biblioteca. |
| ARC-010 | P1 | Workers de jobs e progresso são threads destacadas; `jobs.rs:309-550`, `620-644`. | Falhas de spawn/lifecycle e fechamento não coordenado; progresso preso. | Runtime supervisionado, handles registrados e encerramento estruturado. |
| ARC-011 | P1 | Reserva/tokens de jobs são voláteis, enquanto estado está no banco; `jobs.rs:24-32`, `197-232`. | Divergência após panic/restart e ação que parece presa. | Scheduler orientado ao catálogo com lease, heartbeat e transições atômicas. |
| ARC-012 | P1 | Diversos erros de persistência são descartados com `let _`, `.ok()` ou fallback; exemplos em `jobs.rs:286-306` e `library.rs:98-132`. | Interface informa conclusão sem prova de que o estado foi gravado. | Classificar operações críticas; propagar ou registrar falha durável e compensar. |
| ARC-013 | P1 | Espera por permit usa `Condvar` sem cancelamento nem timeout; `process.rs:103-111`, `resource.rs:28-44`. | Cancelamento não atende 2 s quando fila está ocupada. | Aquisição cancelável, deadline e fairness mensurável. |
| ARC-014 | P1 | Cancelamento de insight é um `AtomicBool` global e cada nova execução o redefine; `insights.rs:14,78-85`. | Uma execução cancela ou reativa outra. | Token por operação e exclusão/coalescência explícita por escopo. |
| ARC-015 | P1 | Insight completo materializa todas as linhas em `Vec`; `insights.rs:116-147`. | Memória cresce com o catálogo. | Agregação SQL/streaming com memória O(1) ou limitada. |
| ARC-016 | P1 | Fingerprint de insights usa apenas contagem e `MAX(created_at)`; `insights.rs:93-96`. | Tags, favoritos, proteção, GPS e correções podem servir cache obsoleto. | Versões/revisões por domínio incluídas na chave. |
| ARC-017 | P1 | Cache global `thumbs` não possui limite/TTL; `Gallery.tsx:33`, `841-873`. | Crescimento por toda a sessão e retenção de estado obsoleto. | LRU limitado e invalidado por versão; URL sem payload retido. |
| ARC-018 | P1 | Paginação acumula todos os assets carregados na sessão; `Gallery.tsx:136-164`. | Scroll longo anula parte do benefício da virtualização. | Windowing também do modelo, ou teto/páginas descartáveis com âncora. |
| ARC-019 | P1 | Effects usam flag `live`, mas trabalho backend continua; `Gallery.tsx:599`, `916-940`. | Navegação rápida cria trabalho inútil e filas; só a resposta é descartada. | IDs/tokens de operação, cancelamento/coalescência ponta a ponta. |
| ARC-020 | P1 | Elemento de vídeo não executa `pause`, remove `src` e `load` no cleanup; `Gallery.tsx:973-975`. | Requisições antigas podem sobreviver à troca e competir por memória/I/O. | Componente de mídia com lifecycle e telemetria explícitos. |
| ARC-021 | P1 | Relatórios acumulam todos os eventos antes de escrever; `events.rs:53-99`; planos de duplicata fazem o mesmo. | Memória proporcional ao histórico. | Cursor → serializador streaming → arquivo temporário atômico. |
| ARC-022 | P1 | Configuração usa `fs::write`; `lib.rs:125-130,246-250`. | Arquivo truncado em interrupção pode perder a biblioteca configurada. | Escrita atômica, fsync e recuperação do predecessor. |
| ARC-023 | P1 | CSP está desabilitada (`csp: null`) em `tauri.conf.json`. | Aumenta impacto de injeção no WebView e protocolo local. | CSP mínima explícita e teste de capacidades. |
| ARC-024 | P1 | Sanitização de diagnóstico é heurística por tokens/extensões; `process.rs:146-165`, `events.rs:118-143`. | Caminhos/PII com formatos inesperados podem escapar. | Eventos estruturados sem dados sensíveis na origem; allowlist no export. |
| ARC-025 | P2 | Cerca de 135 usos de `catalog::open`; políticas de conexão, leitura/escrita e transação ficam distribuídas. | Contenção, regras divergentes e testes difíceis. | `CatalogService` com leitores limitados e escritor serializado. |
| ARC-026 | P2 | `catalog::open` mistura conexão, criação de schema e migrações em um bloco extenso; `catalog.rs:17-528`. | Migração acoplada ao caminho operacional e manutenção arriscada. | Migrator versionado no startup e fábrica de conexões sem DDL. |
| ARC-027 | P2 | Estados continuam espalhados em strings SQL apesar de enums parciais; `jobs.rs` e `engine.rs`. | Transições inválidas e diferenças entre UI/banco. | Máquina de estados central e constraints/triggers de invariantes. |
| ARC-028 | P2 | `lib.rs`, `engine.rs`, `App.tsx` e `Gallery.tsx` concentram composição e regras demais. | “God modules”, revisão difícil e regressões transversais. | Separar casos de uso, portas/adaptadores e componentes por jornada. |
| ARC-029 | P2 | Dashboard dispara insight amostral e refresh completo ao montar; `App.tsx:463-467`. | Trabalho invisível compete com primeira interação. | Snapshot imediato; refinamento somente pelo scheduler e sob orçamento. |
| ARC-030 | P2 | Polling coexistente em 350 ms, 1 s, 2 s, 5 s e threads de eventos. | Amplificação de consultas e estados concorrentes. | Barramento de eventos + polling adaptativo único como fallback. |
| ARC-031 | P2 | Métrica de heartbeat em intervalo de 5 s usa alerta somente após 15 s; `App.tsx:93-98`, `diagnostics.rs:116`. | Não atende o contrato de detecção e confunde atraso com crash. | SLO de 5 s, long-task telemetry e estados `healthy/degraded/stalled`. |
| ARC-032 | P2 | `INITIALIZED` guarda paths não canônicos por toda a vida; `catalog.rs:9,21-28`. | Alias de caminho repete migração; mapa nunca libera biblioteca. | ID canônico de biblioteca e lifecycle do serviço. |
| ARC-033 | P2 | Seleções em lote permitem até 5.000 IDs por IPC/SQL; `lib.rs:1148`, `duplicates.rs:75`. | Payload e statement grandes; trabalho não retomável. | Operação por filtro/selection token e processamento paginado. |
| ARC-034 | P2 | Testes de “500 trocas” provam descarte lógico, não memória da árvore nem cancelamento backend; `Gallery.test.tsx`. | Gate verde sem cobrir a regressão real. | E2E instrumentado no executável e endurance com orçamento. |
| ARC-035 | P2 | Smoke valida abertura, arquivos e encerramento curto, sem navegação/vídeo/jobs concorrentes. | Pacote quebrado passa distribuição. | Smoke funcional automatizado e soak no Windows mínimo. |
| ARC-036 | P2 | CI não executa Clippy, instalador, smoke, budgets ou testes do protocolo hostil; `.github/workflows/ci.yml`. | Documentação promete gates que o branch protection não garante. | Pipeline obrigatório em camadas e artefatos de evidência. |
| ARC-037 | P2 | Documentos de teste e referências de release estão desatualizados (`TESTING.md` ainda era 0.2). | Critério de pronto ambíguo. | Fonte única versionada e links correntes. |
| ARC-038 | P2 | Alterações de catálogo e configuração não compartilham uma unit of work recuperável. | Banco e `library.json` divergem. | Journal de intenção/cutover idempotente. |
| ARC-039 | P3 | Dependências usam faixas sem política documentada de atualização/SBOM. | Builds variam e vulnerabilidades chegam sem triagem consistente. | Lockfiles obrigatórios, SBOM, hashes e cadência de atualização. |
| ARC-040 | P3 | Não há suíte explícita de acessibilidade, contraste, teclado e escala de fonte. | Produto funcional pode permanecer pouco utilizável. | Gate WCAG aplicável ao desktop e roteiro assistivo. |

## Pontos positivos que devem ser preservados

- Fontes não são deliberadamente alteradas pelos fluxos normais.
- Cópia e promoção possuem primitives de hash e temporário verificado.
- Decodificação Rust já define limites de pixels e alocação.
- Fotos deixaram de usar a rota do original no WebView.
- Galeria usa cursor e virtualização visual.
- Existe lock exclusivo de biblioteca e WAL/foreign keys.
- Filas, jobs e parte do progresso estão persistidos.
- Diagnóstico exportado já evita consultar nomes, caminhos e GPS diretamente.

Esses acertos não neutralizam os achados; eles são restrições para a remediação, evitando reescrever o produto do zero.
