# Roadmap do Lumina

O Lumina preserva quatro inegociáveis em toda entrega: fontes permanecem somente leitura; nenhuma proteção é declarada sem verificação; todo trabalho longo é retomável e observável; a galeria continua responsiva durante processamento.

## Concluído — 0.18.5: estabilidade em catálogo real

- Pipeline de metadados limitado por lotes, backpressure de previews e decodificação isolada.
- Filas duráveis, retomada após interrupção, diagnósticos sanitizados e cache reconstruível invisível.
- Galeria virtualizada, preview HD progressivo, vídeo fluido, inspeção, comparação, tags, favoritos e organização local.
- Homologação do usuário com aproximadamente 9.300 arquivos sem novo travamento.

O histórico detalhado permanece em `RELEASE-0.18-BETA.md` e nos hotfixes 0.18.1–0.18.5.

## Entrega atual — 0.19: operação e curadoria

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

## Próximo — 0.20: automação assistida

- Regras de curadoria salvas e filas de revisão configuráveis.
- Busca combinada por data, câmera, local, pessoa, tag e qualidade técnica.
- Comparação orientada a sequências e escolha assistida, sempre explicável e reversível.
- Perfis explícitos de consumo para bateria, CPU e disco.

## Depois — 1.0: prontidão de produção

- Telemetria local longitudinal de SLOs, migração e recuperação validadas em múltiplos dispositivos.
- Assinatura e distribuição do instalador, política de atualização e compatibilidade documentada.
- Fechamento dos critérios de segurança, acessibilidade, desempenho e suporte definidos para produção.
