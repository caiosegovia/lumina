# Arquitetura do Lumina 0.15

```text
React views
    ↓ DTOs tipados
Tauri commands
    ↓ casos de uso
sync | review | duplicates | health | metadata
    ↓
catalog + jobs + media + diagnostics + storage/process
```

## Regras

1. Operações longas são jobs persistidos; chamadas de UI não executam varreduras bloqueantes.
2. Consultas de tela são paginadas e não realizam N+1 por mídia.
3. Toda mutação pessoal registra edição suficiente para desfazer.
4. Estado derivado pode ser reconstruído; estado humano nunca é descartado como cache.
5. Caminhos entram no backend por IDs catalogados sempre que possível.
6. A elegibilidade para limpeza é calculada no backend e repetida no momento da decisão.
7. Migrações são incrementais, idempotentes e executadas em transação.
8. Nenhum módulo novo depende de componentes React ou de detalhes do protocolo Tauri.
9. `metadata` controla extração sob demanda e persistência; a UI nunca executa ferramentas externas diretamente.
10. `media` produz derivados limitados e versionados; originais são somente leitura e vídeo usa leitura por intervalos.
11. `diagnostics` registra apenas eventos sanitizados, com limite de tamanho e sem telemetria externa.
12. A ordenação pertence à consulta do catálogo; cada ordem possui cursor compatível e desempate estável por ID.
13. O frontend informa o conjunto visível, mas `jobs` consolida pedidos e `media` mantém prioridade durável no catálogo.
14. Prefetch é limitado e subordinado ao trabalho interativo; ler uma miniatura nunca gera o arquivo dentro da chamada de UI.
15. Comparação é uma projeção somente leitura de duas mídias catalogadas e não compartilha autoridade com decisões de limpeza.
16. Abrir uma localização resolve o caminho por ID no backend e exige ação explícita do usuário.

## Fluxo de miniaturas invisíveis

```text
viewport virtualizado ── prioridade 180 ─┐
prefetch curto ───────── prioridade 60 ──┼─> JobManager (deduplicação em memória)
abertura direta ──────── prioridade 200 ─┘          │
                                                    v
                                      work_queue persistente
                                                    │
                                      worker de baixa concorrência
                                                    │
                                      cache versionado reconstruível
```

O reinício converte trabalhos interrompidos em pendentes, elimina duplicatas legadas e conclui registros cujo derivado válido já existe. Nenhum modal é necessário para essa manutenção.

## Estratégia de entrega

Cada bloco entra na mesma branch beta com testes próprios. Uma versão só recebe sufixo beta e é indicada para o dispositivo oficial quando todos os gates da especificação estiverem verdes.
