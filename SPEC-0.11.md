# Lumina 0.11 — painel progressivo e inventário técnico

Esta especificação é o contrato integral aprovado para a versão 0.11. A versão não será considerada concluída por uma implementação parcial.

## Requisitos obrigatórios

1. Visão geral em duas etapas: snapshot imediato e atualização completa em background.
2. Instrumentação separada de catálogo, rollups, discos, insights, transporte e atualização.
3. Snapshot versionado, persistente, descartável e reconstruível.
4. Invalidação incremental por importação, proteção, edição e estado de fonte.
5. Cards independentes: falha ou lentidão de uma seção não bloqueia as demais.
6. Big numbers com contexto, comparação e ação.
7. Gráficos de evolução e composição combinados com rankings analíticos.
8. Armazenamento por tipo, formato, ano, câmera e fonte.
9. Proteção por quantidade, bytes, período e origem.
10. Duplicidade em duas visões: organização e espaço potencial conservador.
11. Modelo preparado para futura decisão por ocorrência, sem exclusão na 0.11.
12. Seção de insights independente de números e gráficos.
13. Insights com causa, impacto, confiança, prioridade e ação contextual.
14. Registro técnico central de fotos, RAWs, vídeos, contêineres e codecs.
15. Detecção por conteúdo e registro de divergência entre extensão e conteúdo.
16. Níveis de suporte: completo, parcial, preservação, desconhecido e inválido.
17. Formato válido sem preview não pode ser tratado como arquivo corrompido.
18. Enriquecimento progressivo do acervo existente em job persistente e retomável.
19. Migração segura sem alterar origens nem bytes consolidados.
20. Navegação contextual de números, gráficos e insights para filtros da galeria.
21. Benchmark frio/quente com 100 mil e 500 mil registros e concorrência ativa.
22. Entrega com testes, relatório comparativo, portável, EXE, MSI e smoke isolado.

## Metas

- Snapshot quente: até 300 ms.
- Primeira resposta com cache frio: até 800 ms.
- Interação durante atualização: até 100 ms.
- Consulta principal com 100 mil itens: p95 até 100 ms.
- Consulta principal com 500 mil itens: p95 até 300 ms.
- Divergência entre snapshot e catálogo: zero.
- Alteração ou exclusão de arquivo de origem: zero.

## Limites

Não fazem parte desta versão: exclusão de duplicatas, reconhecimento facial, similaridade visual, Google Takeout, edição destrutiva, servidor e acesso remoto.
