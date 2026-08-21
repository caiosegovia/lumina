# Lumina 0.12 — inventário profundo e inteligência de storage

Este documento é o contrato integral da versão 0.12. A entrega não é considerada concluída por uma implementação parcial.

## Inventário técnico

1. Diferenciar registro básico de inventário técnico completo.
2. Reenfileirar registros incompletos, inclusive vídeos sem contêiner ou codec.
3. Registrar vídeo e áudio: contêiner, codecs, resolução, FPS, bitrate, duração e pixel format.
4. Registrar fotografia e RAW: dimensões, lente, ISO, abertura, exposição, distância focal, orientação, perfil de cor, GPS e disponibilidade de preview.
5. Separar arquivo corrompido, decoder incompatível, preview ausente, metadados incompletos e formato de preservação.
6. Preservar TIFF e outros formatos válidos sem preview; ausência de decoder não significa corrupção.
7. Tornar o enriquecimento persistente, retomável, cancelável, idempotente e de baixa prioridade.

## Storage e proteção

8. Exibir capacidade, uso e espaço livre do acervo e da réplica.
9. Separar uso total do disco, bytes administrados pelo Lumina, cache e temporários.
10. Calcular volume pendente de proteção, margem depois da réplica e reserva de segurança.
11. Estimar quantas mídias adicionais cabem usando média e percentil de tamanho.
12. Produzir composição por tipo, formato, ano, câmera, fonte e proteção.
13. Produzir crescimento mensal/anual e projeção conservadora de capacidade.

## Experiência

14. Dashboard progressivo com snapshot imediato e seções atualizadas independentemente.
15. Big numbers com contexto, comparação, estado e ação contextual.
16. Seções distintas para resumo executivo, storage, composição, inventário e insights.
17. Gráficos e rankings devem ter proporção visual, legenda, tooltip e navegação para filtros.
18. Mostrar cobertura de miniaturas, metadados, codecs, integridade e proteção.
19. Ocultar fontes internas de manutenção e traduzir estados técnicos para linguagem de usuário.
20. Manter a interface responsiva enquanto miniaturas e enriquecimento rodam.

## Qualidade e entrega

21. Instrumentar backend, transporte e renderização; manter benchmarks de 100 mil e 500 mil registros.
22. Entregar testes, relatório comparativo, checklist, portátil, EXE, MSI, hashes e smoke isolado.

## Metas

- Snapshot inicial: p95 até 300 ms.
- Dashboard completo em segundo plano: até 1,5 s em 100 mil e 4 s em 500 mil registros no perfil de teste sem otimização.
- Interação durante background: p95 até 100 ms.
- Divergência entre cards e catálogo: zero.
- Originais ou fontes alterados pelo inventário: zero.
