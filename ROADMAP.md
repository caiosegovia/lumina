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

## Entrega atual — 0.20: lugares e curadoria privada

- Resolução local de coordenadas em cidades, com fallback honesto para região aproximada.
- Cache derivado e idempotente, correções manuais persistentes e nenhuma escrita em EXIF.
- Contadores e ações de lugares em Descoberta, abertura do conjunto e busca textual integrada na Galeria.
- Migração v16, testes de privacidade e regressão completa em instaladores Windows.

## Próximo — 0.21: automação assistida

- Regras de curadoria salvas e filas de revisão configuráveis.
- Filtros explícitos por lugar, combinações salvas e navegação de viagens aprimorada.
- Comparação orientada a sequências e escolha assistida, sempre explicável e reversível.
- Perfis explícitos de consumo para bateria, CPU e disco e expansão versionada da base geográfica.

## Depois — 1.0: prontidão de produção

- Telemetria local longitudinal de SLOs, migração e recuperação validadas em múltiplos dispositivos.
- Assinatura e distribuição do instalador, política de atualização e compatibilidade documentada.
- Fechamento dos critérios de segurança, acessibilidade, desempenho e suporte definidos para produção.
