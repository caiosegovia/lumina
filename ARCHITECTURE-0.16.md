# Arquitetura operacional — Lumina 0.16

```mermaid
flowchart LR
  DB[(SQLite jobs)] --> API[list_jobs]
  API --> JS[jobState.ts]
  JS --> A[Em andamento]
  JS --> B[Precisa de ação]
  JS --> C[Histórico]
  A --> P[Polling 1 s]
  B --> I[Próximo passo explícito]
  C --> H[Histórico recolhível]
  H --> Q[Polling 5 s]
```

O backend continua sendo a autoridade sobre transições e persistência. O frontend centraliza apenas classificação, texto e cadência de atualização; não inventa estados nem promove jobs.

```mermaid
flowchart LR
  L[Lista compacta] --> F[Filtro e ordenação]
  F --> R[50 grupos renderizados]
  R -->|clique| E[Grupo expandido]
  E --> T[Preview e ocorrências]
  E --> D[Decisões persistentes]
  D --> S{Réplica verificada?}
  S -->|não| X[Candidatura bloqueada]
  S -->|sim| Y[Candidatura permitida]
```

Previews de duplicatas deixam de ser custo inicial da página. As garantias de decisão permanecem no backend e independem da apresentação recolhida ou expandida.
