# Roadmap do Lumina

## Próximo ciclo após a 0.10.0

### Desempenho da Visão geral

- Instrumentar tempos de catálogo, disco, rollups e renderização.
- Servir primeiro o último snapshot do painel e atualizar dados em segundo plano.
- Separar disponibilidade de espaço/volumes das consultas do catálogo.
- Invalidar apenas os agregados afetados por importação, proteção ou edição.
- Definir orçamento de abertura: conteúdo útil inicial abaixo de 300 ms e atualização completa sem bloquear navegação.

### Insights acionáveis

- Crescimento da biblioteca em 30 dias, 12 meses e por ano.
- Maiores anos, formatos, câmeras e origens por espaço ocupado.
- Cobertura de proteção por período e por fonte.
- Arquivos com datas suspeitas, metadados ausentes e miniaturas pendentes.
- Duplicidade por origem com espaço potencial, sem sugerir exclusão automática.
- Recomendações ordenadas por risco, impacto e ação disponível.

### Critérios de aceite

- Benchmark frio e quente registrado com biblioteca de 100 mil itens.
- Nenhuma consulta de painel no thread da interface.
- Cache consistente depois de importar, proteger, reconectar fonte e editar data.
- Cada insight informa causa, quantidade, espaço ou risco e oferece uma ação compreensível.
