# Roadmap do Lumina após a 0.12

Este roadmap traduz a visão de produto em ciclos verificáveis. Datas são definidas somente quando capacidade e escopo estiverem confirmados; segurança, integridade e retomada não são negociadas para cumprir prazo.

## Agora — 0.13: robustez de produto

Objetivo: transformar a base tecnicamente segura em uma experiência previsível sob uso real.

- Tornar miniaturas transparentes: sob demanda para itens visíveis, prefetch curto e preenchimento silencioso em background.
- Recuperar filas internas sem modal, eliminar trabalho duplicado e priorizar interação.
- Separar claramente trabalhos iniciados pelo usuário de manutenção reconstruível.
- Cobrir restart, falta de espaço, volumes offline e falhas de ferramentas em E2E do aplicativo empacotado.
- Alinhar CI ao gate de release: formato, testes, build, Clippy, auditorias e smoke.
- Produzir diagnóstico exportável sem caminhos pessoais ou metadados sensíveis.
- Reduzir os maiores pontos de acoplamento no backend e frontend sem reescrita ampla.

Critérios de saída:

- Nenhuma manutenção de cache exige decisão do usuário ao abrir o app.
- Uma tarefa efetiva de miniatura por mídia, independentemente da importação que a originou.
- Itens visíveis recebem prioridade e a galeria permanece interativa sob carga.
- Fechar e reabrir não perde trabalho nem cria trabalho redundante.
- Gate automatizado e roteiro funcional passam em pacote portátil, EXE e MSI.

## Depois — 0.14: revisão segura de duplicatas

Objetivo: ajudar a decidir, sem transformar estimativas em exclusão automática.

- Agrupar ocorrências idênticas e explicar origem, proteção e espaço potencial.
- Persistir decisões por ocorrência: manter, revisar e candidata a remoção.
- Exigir réplica verificada e confirmação explícita antes de qualquer remoção futura.
- Oferecer simulação, relatório e plano de recuperação antes de executar.
- Manter fontes somente leitura por padrão; qualquer exceção será um modo separado e deliberado.

O ciclo foi ampliado para a beta de revisão e descoberta descrita em [SPEC-0.14-BETA.md](SPEC-0.14-BETA.md): sincronização incremental, Central de Revisão, visualizador avançado, álbuns inteligentes e saúde operacional fazem parte do mesmo gate de produto.

A beta.4 acrescenta o fechamento do ciclo de homologação: preview HD progressivo, EXIF sob demanda, observabilidade de crashes, estados explícitos de duplicatas, progresso real de reparos e redesign do sistema de interação.

## Próxima evolução de UX — galeria unificada

Feedback registrado após a beta.4:

- Redesenhar a visualização em lista para oferecer hierarquia, densidade e leitura de metadados compatíveis com a grade.
- Substituir a dependência da barra lateral por uma faixa fixa de segmentação dentro da galeria.
- Fazer a segmentação conviver com grade e lista, preservando a maior área possível para as mídias.
- Manter filtros, agregadores, busca, ordenação, seleção e contagens no mesmo contexto em ambas as visualizações.
- Preservar posição, seleção e filtros ao alternar entre grade e lista.
- Tratar responsividade, navegação por teclado, foco, estados vazios e grandes catálogos como critérios de aceite do redesign.

Essa evolução será especificada e prototipada depois da homologação da beta.4; ela não altera o escopo binário desta candidata.

Situação em `0.15.0-beta.1`:

- Homologados: workspace dividido, inspetor embutido, segmentação fixa, lista operacional responsiva, sequência recolhível e comportamento com grande quantidade de arquivos.

## Próximas entregas — ciclo 0.15

### Beta.2 — produtividade da galeria

- Ordenação persistente no backend, com paginação por cursor coerente para data, nome e tamanho.
- Densidade confortável e compacta específica para a lista.
- Seleção por intervalo, selecionar itens carregados e ações em lote mais claras.
- Cabeçalhos acionáveis com direção de ordenação e descrição acessível.
- Preservação de posição, seleção e item inspecionado durante as mudanças de representação.

Critérios de saída:

- Nenhuma ordenação é simulada apenas sobre a página já carregada.
- Grade e lista retornam exatamente a mesma ordem e conjunto.
- Operações em lote informam escopo, resultado e possibilidade de desfazer.
- Testes cobrem troca de modo, paginação, seleção por intervalo e catálogos grandes.

### Beta.3 — inspeção e comparação

- Organizar metadados em seções recolhíveis: captura, arquivo, mídia, localização e catálogo.
- Copiar valores e abrir a localização do arquivo com ação explícita.
- Comparação lado a lado acionável a partir da galeria e integrada às duplicatas.
- Sincronizar zoom, navegação e metadados das duas mídias quando fizer sentido.
- Refinar feedback de preview imediato, geração HD e indisponibilidade.

Critérios de saída:

- Navegação nunca exibe preview ou metadados da mídia anterior.
- Comparação não infere duplicidade nem permite remoção automática.
- Valores ausentes são explicados e seções mantêm sua preferência.

### Beta.4 — trabalho invisível e confiabilidade

- Priorizar miniaturas do viewport e aplicar prefetch curto na direção da navegação.
- Consolidar trabalhos duplicados e retomar manutenção reconstruível sem modal.
- Controles de impacto para CPU, disco e bateria durante processamento em segundo plano.
- Histórico local de sincronização, reparo e falhas acionáveis.
- Diagnóstico exportável sanitizado e roteiro de recuperação validado.

Critérios de saída:

- Abrir o aplicativo nunca exige decisão sobre cache reconstruível.
- A interação continua prioritária sob carga.
- Reinício, fonte offline, pouco espaço e falha de ferramenta passam no aplicativo empacotado.

### Fechamento 0.15

- Regressão completa da biblioteca, benchmark de grande volume e validação no dispositivo oficial.
- Revisão de acessibilidade em 100%, 125% e 150% de escala do Windows.
- Atualização da baseline visual com capturas aprovadas.
- MSI, instalador, portátil, hashes, documentação e release publicados juntos.

## Exploração — 0.15+: descoberta e portabilidade

- Similaridade visual local e explicável.
- Pessoas/rostos com processamento local, consentimento e controles de privacidade.
- Ingestão assistida de Google Takeout e exportações equivalentes.
- Confirmação de backup remoto por integração verificável, sem inferir upload por pasta sincronizada.
- Portabilidade de catálogo, backup de configuração e migração entre computadores.
- Acesso remoto somente depois de um modelo de ameaças e autenticação adequados.

## Trilhas contínuas

- Compatibilidade real de formatos e fixtures RAW/vídeo.
- Acessibilidade, linguagem clara e operações reversíveis.
- Performance medida em catálogos e arquivos reais: latência, throughput, CPU e memória.
- Segurança de dependências, protocolo local, caminhos e processos externos.
- Documentação de decisões arquiteturais e redução incremental de módulos grandes.

## Fora de compromisso atual

Aplicativo móvel, edição de imagem, rede social, nuvem própria e exclusão autônoma não fazem parte do plano vigente.
