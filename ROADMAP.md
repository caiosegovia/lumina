# Roadmap do Lumina

O Lumina preserva quatro inegociáveis em toda entrega: fontes permanecem somente leitura; nenhuma proteção é declarada sem verificação; todo trabalho longo é retomável e observável; a galeria continua responsiva durante processamento.

## Concluído — 0.18.5: estabilidade em catálogo real

- Pipeline de metadados limitado por lotes, backpressure de previews e decodificação isolada.
- Filas duráveis, retomada após interrupção, diagnósticos sanitizados e cache reconstruível invisível.
- Galeria virtualizada, preview HD progressivo, vídeo fluido, inspeção, comparação, tags, favoritos e organização local.
- Homologação do usuário com aproximadamente 9.300 arquivos sem novo travamento.

O histórico detalhado permanece em `RELEASE-0.18-BETA.md` e nos hotfixes 0.18.1–0.18.5.

## Concluído — 0.19: operação e curadoria

### Atividade compreensível

- Separar execução, atenção e histórico.
- Diferenciar espera legítima de job ativo sem heartbeat.
- Exibir última evolução, próxima ação, progresso, pausa, retomada, cancelamento e repetição de falhas.
- Mostrar manutenção de metadados e previews como trabalho silencioso, sem bloquear a abertura.

### Duplicatas progressivas e seguras

- Listagem compacta com pills de quantidade, espaço, proteção e decisão.
- Buscar ocorrências somente quando o grupo for expandido, evitando N+1 no catálogo grande.
- Seleção múltipla e decisão transacional em lote.
- Bloquear candidatas à remoção sem réplica verificada; manter simulação e relatório sem excluir originais.

### Produtividade da galeria

- Preservar grade/lista virtualizadas, segmentação fixa, filtros agregadores e seleção destacada.
- Manter inspeção embutida, preview HD, zoom, tela cheia, vídeo e comparação lado a lado.
- Manter tags, favoritos, pessoas e álbuns reversíveis no catálogo.

### Proteção operacional

- Apresentar pendentes, em cópia, protegidos, falhas, volume restante e cobertura.
- Atualizar o estado enquanto a proteção executa e tornar falhas acionáveis na Atividade.

### Gate da beta

- Testes frontend e Rust, TypeScript/Vite, formatação, Clippy, build release e smoke do pacote portátil.
- Instalação NSIS/MSI e roteiro manual em catálogo de homologação com carga concorrente.

## Concluído — 0.20: lugares e curadoria privada

- Resolução local de coordenadas em cidades, com fallback honesto para região aproximada.
- Cache derivado e idempotente, correções manuais persistentes e nenhuma escrita em EXIF.
- Contadores e ações de lugares em Descoberta, abertura do conjunto e busca textual integrada na Galeria.
- Migração v16, testes de privacidade e regressão completa em instaladores Windows.

## Concluído — 0.21: tempo, lugares e bursts confiáveis

- Corrigir o falso UTC de datas EXIF e tornar sua proveniência visível.
- Usar a base geográfica mundial offline embarcada, filtro exato e viagens legíveis.
- Detectar bursts por tempo, equipamento e coerência visual.
- Favoritar a melhor candidata e enviar alternativas para revisão sem exclusão.

## Concluído — 0.22: lugares e automação confiáveis

- Metadados geográficos ricos, cache versionado e refinamento por bairro/sublocalização.
- Seleção correta no Explorer, feedback visível e cópia de coordenadas.
- Comparação de até quatro candidatas e ações seguras em bursts.
- Perfis explícitos de I/O e regras persistentes de curadoria.

## Publicada, mas reprovada em homologação — 0.22.1

- Remover originais fotográficos do WebView e manter miniatura → preview limitado.
- Telemetria por preview com bytes, dimensões e variação de memória.
- Motor isolado de insights com amostra estratificada e análise completa por acervo, ano ou mês.
- Cache versionado, cobertura explícita, cancelamento e cards acionáveis.
- Localização v3.1 sem misturar campos nativos confiáveis com fragmentos inferidos.
- Regressão integral da galeria e pacote homologável Windows.

A automação e o smoke curto foram aprovados, porém a homologação prolongada encontrou crescimento abrupto da árvore de processos e encerramento inesperado. A causa principal confirmada é a leitura de vídeo sem intervalo limitado; a auditoria também encontrou outros caminhos sem orçamento. O histórico permanece preservado, mas a versão não é candidata de produção.

## Em preparação — 0.22.2: resiliência arquitetural

- Limitar protocolo de vídeo, saídas de ferramentas, arquivos individuais e exportações.
- Adotar coordenador global de recursos e runtime supervisionado.
- Tornar jobs duráveis por lease/heartbeat, com cancelamento e shutdown prováveis.
- Corrigir atomicidade de configuração, alteração de réplica e migração SQLite.
- Limitar lifecycle e caches da galeria; medir a árvore completa de processos.
- Aplicar CSP, logs estruturados e diagnóstico privado por construção.
- Homologar em 100 mil registros, arquivo de 64 GiB, equipamento mínimo e soak de 2 horas.

Escopo, ordem e gates estão em `NFR-0.22.2-RESILIENCE.md`, `ARCHITECTURE-AUDIT-0.22.2.md`, `REMEDIATION-0.22.2.md` e `TRACEABILITY-0.22.2.md`.

## Congelado até o aceite da 0.22.2 — 0.23: produtividade e produção assistida

- Revisão tipográfica integral: adotar uma família moderna e legível, com fallback local previsível, pesos consistentes e renderização uniforme no Windows/WebView2.
- Consolidar tokens tipográficos para títulos, seções, cards, metadados, pills, botões e textos auxiliares, eliminando tamanhos e estilos isolados.
- Revisar hierarquia, espaçamento, altura de linha, contraste, truncamento e comportamento em diferentes escalas de tela e DPI.
- Validar visualmente todas as seções e estados — carregamento, vazio, erro, seleção, zoom e comparação — antes de publicar a próxima candidata.
- Comparação de períodos, insights por lugar/equipamento/tag e curadoria orientada.
- Duplicatas com confiança editorial e revisão compacta refinada.
- Jobs idempotentes com estados mais claros, diagnóstico guiado e retomada provada.
- Telemetria longitudinal e diagnóstico comparativo entre dispositivos.
- Políticas de bateria e agendamento por janela de uso.
- Mapa privado opcional e pacotes geográficos regionais atualizáveis.

## Depois — 1.0: prontidão de produção

- Telemetria local longitudinal de SLOs, migração e recuperação validadas em múltiplos dispositivos.
- Assinatura e distribuição do instalador, política de atualização e compatibilidade documentada.
- Fechamento dos critérios de segurança, acessibilidade, desempenho e suporte definidos para produção.
