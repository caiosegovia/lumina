# Lumina 0.13 — robustez de produto

Esta especificação define o ciclo iniciado depois da 0.12. A implementação pode ser entregue em incrementos, mas a versão 0.13 só será concluída quando todos os critérios obrigatórios forem validados.

## Experiência e filas

1. Miniaturas são cache reconstruível e sua recuperação não exige confirmação.
2. Itens visíveis na galeria têm prioridade sobre preenchimento em massa.
3. Reinício converte trabalho interno interrompido em trabalho pendente e o retoma silenciosamente.
4. Existe no máximo uma tarefa efetiva de miniatura por mídia.
5. Tarefas antigas de diferentes importações são normalizadas sem perder miniaturas prontas.
6. Trabalhos internos nunca aparecem como importação interrompida.
7. Importação, proteção e inventário continuam explicitamente controláveis pelo usuário.

## Confiabilidade

8. O aplicativo empacotado possui cenários E2E de restart, volume offline, falta de espaço e ferramenta externa indisponível.
9. Diagnóstico exportável omite ou anonimiza caminhos e metadados pessoais.
10. Estados persistidos possuem transições tipadas, invariantes e recuperação testada.
11. Falhas recuperáveis não encerram o aplicativo nem deixam itens indefinidamente em processamento.

## Qualidade e arquitetura

12. O CI executa o mesmo núcleo do gate de release.
13. Rust passa por formatação, Clippy, testes e auditoria de dependências.
14. Frontend passa por lint, testes, build e auditoria de dependências.
15. Os módulos de maior risco recebem fronteiras menores orientadas a casos de uso, sem reescrita ampla.
16. Benchmarks registram latência, throughput, memória e CPU em cenário representativo.

## Critérios de aceite do primeiro incremento

- Catálogo legado com tarefas duplicadas é normalizado automaticamente.
- Miniatura em processamento retorna para pendente após reinício.
- Miniatura pronta não é regenerada.
- Solicitação sob demanda eleva prioridade sem criar nova tarefa.
- Modal de recuperação lista somente trabalhos que exigem decisão humana.
- Suítes, benchmark, build otimizada e smoke isolado permanecem verdes.
