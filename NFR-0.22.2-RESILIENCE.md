# Contrato não funcional — Lumina 0.22.2

Status: **aprovado para implementação**. Este documento é o contrato de aceite da estabilização 0.22.2. Em caso de conflito, ele prevalece sobre metas informais de releases anteriores.

## Plataforma e escala suportadas

| ID | Requisito |
|---|---|
| NFR-CAP-01 | Windows 10 22H2 e Windows 11, somente x64. |
| NFR-CAP-02 | Equipamento mínimo: 8 GiB de RAM e 4 processadores lógicos. |
| NFR-CAP-03 | Catálogo suportado: até 100.000 mídias. |
| NFR-CAP-04 | Arquivo individual suportado: até 64 GiB (`68.719.476.736` bytes). Arquivos maiores devem ser inventariados como não suportados, com motivo acionável, e nunca decodificados, copiados ou carregados automaticamente. |
| NFR-CAP-05 | O tamanho do arquivo nunca determina diretamente o tamanho de uma alocação. Leituras, respostas e processamento usam blocos limitados. |

## Orçamentos de recursos

Os limites de memória abrangem a **árvore inteira de processos**: processo Rust, WebView2 e ferramentas filhas. O valor absoluto inclui memória residente privada e compartilhada atribuível à árvore, usando a mesma metodologia em todas as medições.

| ID | Cenário | Meta de memória |
|---|---|---:|
| NFR-MEM-01 | Aplicativo ocioso, catálogo aberto | menor que 250 MiB |
| NFR-MEM-02 | Navegação e inspeção de fotografias | menor que 600 MiB |
| NFR-MEM-03 | Reprodução e busca em vídeo | menor que 800 MiB |
| NFR-MEM-04 | Importação, metadados, proteção ou manutenção pesada | menor que 1 GiB |
| NFR-MEM-05 | Pico transitório absoluto | menor que 1,25 GiB |
| NFR-MEM-06 | Após 500 trocas de mídia e estabilização | no máximo 100 MiB acima da linha de base |
| NFR-MEM-07 | Alocação de uma resposta de protocolo local | no máximo 4 MiB |
| NFR-MEM-08 | Saída capturada por processo externo | no máximo 8 MiB por canal; excedente é drenado e truncado com telemetria |

O coordenador deve limitar simultaneamente: um gerador de preview fotográfico, uma extração interativa de metadados, até duas leituras parciais de vídeo e uma operação pesada de insights/manutenção. Trabalho de fundo cede capacidade à interação e reduz concorrência sob pressão de memória.

## Responsividade e cancelamento

| ID | Requisito |
|---|---|
| NFR-LAT-01 | Ação visual do usuário recebe feedback em até 100 ms. |
| NFR-LAT-02 | Troca para miniatura disponível ocorre em até 300 ms no p95. |
| NFR-LAT-03 | Nenhuma tarefa síncrona bloqueia a interface por mais de 500 ms. |
| NFR-LAT-04 | Heartbeat da interface permanece com idade máxima de 5 s durante carga suportada. |
| NFR-CAN-01 | Cancelamento é reconhecido em até 2 s, inclusive enquanto aguarda capacidade. |
| NFR-CAN-02 | Toda operação longa é cancelável, retomável ou explicitamente declarada indivisível e limitada. |
| NFR-CAN-03 | Fechamento interrompe admissão, cancela workers, aguarda término limitado e persiste estado recuperável. Não podem restar threads órfãs. |

## Integridade e durabilidade

| ID | Requisito |
|---|---|
| NFR-DAT-01 | Fontes são somente leitura. Nenhum caminho de análise, preview ou inventário escreve em originais. |
| NFR-DAT-02 | Promoção ao acervo e réplica exige SHA-256 verificado e troca atômica no mesmo volume. |
| NFR-DAT-03 | Alterações correlatas de catálogo são transacionais; erro não deixa estados parcialmente verdadeiros. |
| NFR-DAT-04 | Configuração é gravada por arquivo temporário, sincronização e substituição atômica. |
| NFR-DAT-05 | Migração e snapshot SQLite usam a API de backup consistente e exclusão de escritores; copiar o arquivo principal não é evidência suficiente. |
| NFR-DAT-06 | Reinício após encerramento forçado não duplica efeitos nem perde progresso confirmado. |
| NFR-DAT-07 | Estados e transições possuem uma única máquina de estados tipada e invariantes verificadas no banco. |

## Falhas e degradação

| ID | Requisito |
|---|---|
| NFR-RES-01 | Memória, disco, ferramenta externa ou arquivo hostil não podem encerrar o aplicativo silenciosamente. |
| NFR-RES-02 | Ao atingir 80% do orçamento, suspender prefetch e trabalho de fundo; a 90%, cancelar derivados não essenciais; no limite, rejeitar nova admissão e preservar navegação básica. |
| NFR-RES-03 | Toda falha recebe código estável, componente, operação, correlação e ação recomendada. |
| NFR-RES-04 | Jobs possuem lease/heartbeat durável; lease vencido resulta em estado recuperável, nunca em “processando” indefinidamente. |
| NFR-RES-05 | Tentativas usam máximo explícito, backoff com jitter e fila de falhas permanentes; não existem loops infinitos de retry. |

## Observabilidade e privacidade

| ID | Requisito |
|---|---|
| NFR-OBS-01 | Métricas incluem árvore de processos, memória atual/pico, CPU, I/O, tamanho das filas, operação ativa, WebView e processos externos. |
| NFR-OBS-02 | Logs são serializados por um único escritor, estruturados, rotacionados sem corrida e encerrados com flush. |
| NFR-OBS-03 | Um `operation_id` correlaciona frontend, comando, job, item e processo externo. |
| NFR-OBS-04 | Diagnóstico exportado remove caminhos, nomes, coordenadas, hashes e segredos por construção, com testes de propriedades e fixtures hostis. |
| NFR-OBS-05 | Retenção local é limitada e documentada; o usuário pode exportar e limpar diagnósticos. |

## Segurança e manutenção

| ID | Requisito |
|---|---|
| NFR-SEC-01 | CSP explícita, permissões Tauri mínimas e protocolos locais com validação estrita de método, identificador, tipo, intervalo e raiz canônica. |
| NFR-SEC-02 | Dependências e ferramentas empacotadas possuem versão, licença, hash e verificação automatizada. |
| NFR-MAN-01 | Módulos de domínio não dependem de Tauri; adaptadores de IPC, SQLite, filesystem e processos ficam nas bordas. |
| NFR-MAN-02 | Nenhum módulo de composição concentra regras de galeria, jobs, dashboard e persistência. |
| NFR-MAN-03 | Mudança de schema possui migração transacional, teste de upgrade, rollback/recovery e backup anterior. |

## Gate de aceite

Uma release só pode declarar estes requisitos atendidos com artefato reproduzível: teste automatizado, medição de endurance, relatório do catálogo de 100 mil registros, teste com arquivos esparsos de 64 GiB, instalação limpa e upgrade em dispositivo mínimo. Compilar, abrir a janela ou testar apenas fixtures pequenas não encerra nenhum requisito.
