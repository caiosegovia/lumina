# Arquitetura alvo da 0.14 beta

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

## Estratégia de entrega

Cada bloco entra na mesma branch beta com testes próprios. Uma versão só recebe sufixo beta e é indicada para o dispositivo oficial quando todos os gates da especificação estiverem verdes.
