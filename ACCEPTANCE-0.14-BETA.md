# Matriz de aceite — Lumina 0.14 beta

Esta matriz é o contrato operacional do ciclo. Um item só está entregue quando há comportamento observável, teste automatizado e validação funcional. Build isolado não equivale a entrega.

| Área | Requisito observável | Estado atual | Evidência necessária |
|---|---|---:|---|
| Sincronização | Atualizar uma ou todas as fontes sem duplicar a fonte lógica | Implementado | integração + homologação |
| Sincronização | Classificar novas, alteradas, presentes e ausentes preservando histórico | Implementado | integração + inspeção do catálogo |
| Sincronização | Retomar sincronização após reinício e nunca escrever na fonte | Implementado | E2E restart + snapshot da fonte |
| Revisão | Todas as filas obrigatórias com contagem e destino acionável | Implementado e testado | homologação visual aberta |
| Revisão | Ação rápida, avanço automático e trilha reversível visível | Implementado e testado | integração aprovada |
| Duplicatas | Comparação visual lado a lado com origem, proteção e espaço | Implementado e testado | homologação visual aberta |
| Duplicatas | Decisão persistente por grupo e ocorrência | Implementado e testado | migração + integração aprovadas |
| Duplicatas | Plano/relatório sem exclusão e bloqueio sem réplica | Implementado | integração + inspeção da fonte |
| Visualizador | Foto/vídeo sincronizados, anterior/próxima, atalhos e tela cheia | Implementado e testado | teste de troca foto+metadado aprovado |
| Visualizador | Filmstrip compreensível, zoom e pan | Implementado e testado | homologação visual aberta |
| Visualizador | Metadados completos e atualizados durante navegação | Implementado na beta.4 | EXIF real + persistência + frontend |
| Visualizador | Comparação lado a lado quando aplicável | Implementado em Duplicatas | homologação visual aberta |
| Organização | CRUD completo de visões e álbuns inteligentes | Implementado e testado | build e frontend aprovados |
| Organização | CRUD completo de álbuns manuais | Implementado e testado | build e frontend aprovados |
| Organização | CRUD de tags e ações em lote | Implementado | integração + frontend |
| Saúde | Catálogo, fontes, miniaturas, réplica, filas e ferramentas | Implementado e testado | ExifTool/FFmpeg/FFprobe incluídos |
| Saúde | Progresso uniforme, histórico, reparos e falhas acionáveis | Implementado e testado | restart/offline aprovados |
| Privacidade | Diagnóstico sem caminhos, nomes, hashes, GPS ou conteúdo | Implementado | teste de conteúdo |
| Release | Migração anterior/rollback, restart, offline, espaço e ferramentas | Validado | suíte completa aprovada |
| Release | Benchmark grande e homologação em `D:\Galeria Caio` | Validado tecnicamente | 500 mil itens + app real responsivo |
| Release | EXE, MSI e NSIS beta com hashes | Beta.2 gerada | hashes em `VALIDATION-0.14-BETA.md` |

## Bloqueadores informados na homologação e resolução beta.4

- Preview principal sincronizado: validado pelo responsável do produto na beta.3.
- Crash durante navegação: não reproduzido após contenção de memória da beta.3; beta.4 preserva a correção.
- Zoom sem alta qualidade: resolvido com preview derivado de até 2560 px e teste de dimensão/cache.
- Metadados de Captura vazios: resolvido com ExifTool sob demanda, parser robusto e persistência.
- Filmstrip sem contexto: identificado como **Próximas mídias** e mantido como navegação acionável.
- Botões inconsistentes: design system global aplicado na beta.4.
- Saúde confusa: resumo por prioridade, problemas acionáveis e detalhes técnicos recolhíveis.
- Duplicatas vazias sem explicação: estado agora informa cobertura do catálogo, ocorrências e fontes.
- Reparo pouco claro: progresso numérico, recuperadas e falhas expostos durante execução.

## Evidência nova da beta.4

- Preview HD: geração isolada, limite de 2560 px, reuso do cache e teste automatizado com imagem de 3000 px.
- EXIF sob demanda: câmera, lente, ISO e abertura gravados e relidos em teste de integração.
- Galeria: agregadores e filtros rápidos com teste de regressão.
- Observabilidade: marcador de sessão, encerramento limpo, panic hook, erros frontend/mídia e rotação local.

Nenhum item obrigatório pode ser removido, simplificado ou adiado sem autorização explícita do responsável pelo produto.
