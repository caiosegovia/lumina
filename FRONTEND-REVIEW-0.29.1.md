# Revisão do frontend — 0.29.1-beta.1

## Escopo e método

Revisão das nove seções: Visão geral, Biblioteca, Descobrir, Revisão, Fontes, Duplicatas, Álbuns, Atividade e Proteção. Inspeção de código, interação automatizada em navegador e screenshots nos temas claro/escuro. A busca também passou por 50 consultas no executável extraído do MSI, com catálogo isolado de arquivos gerados, sem mocks de IPC.

## Achados e correções

| Achado | Correção |
| --- | --- |
| O formulário recebia `loading`; regra global exigia `min-height:100vh` | Estado exposto por `aria-busy`; formulário tem altura estável |
| Grade de controles tinha largura mínima incompatível com o inspetor | Busca ocupa linha própria e controles se acomodam na linha seguinte |
| Botão de busca desabilitado durante consultas e quando o texto não mudava | Submissão disponível durante carregamento e repetição após erro |
| Resposta anterior podia chegar entre mudança de filtro e início da próxima consulta | Assinatura da consulta validada junto com geração; limpeza invalida respostas |
| Paginação podia usar cursor da consulta anterior | Cursor só é usado após a primeira página da assinatura atual |
| Página adicional sobrescrevia contador com zero | Mantém contador, resumo e opções da primeira página |
| Visão salva não atualizava texto do campo | Campo sincronizado com filtro aplicado |
| Colunas da grade calculadas pela janela inteira | ResizeObserver acompanha a largura real do canvas |
| Lista extrapolava o painel disponível | Colunas flexíveis e layout compacto por container query |
| Ordenação escondida em telas menores | Controle permanece acessível na linha de ações |
| Botões de seleção escondidos por breakpoint | Barra contextual acomoda os comandos em linhas |
| Fontes extrapolava a área útil de 800 px | Grid de fontes responsivo e conteúdo com largura limitada |
| Proteção extrapolava a área útil de 667 px | Fluxo vertical em janelas menores e quebra segura de caminhos |
| Barra fixa encobria a ação de limpar a busca em janelas baixas | Barra acompanha a rolagem em áreas menores; teste clica no botão sem forçar interação |
| Filtro de cópia mestre aparecia como réplica protegida | Labels distinguem os cinco estados; ausência de GPS também tem pill |
| Polling de jobs podia continuar após desmontagem | Encerramento explícito e descarte de respostas superadas |
| Aviso dizia “sem worker ativo” inferindo isso de data antiga | Texto comunica ausência de atualização, sem concluir travamento |
| Fila e execução misturadas | Aba Aguardando separada de Em execução |
| Sem ação explícita de atualização de Atividades | Consulta manual e indicação de horário; falhas são visíveis |

## Estrutura

`GallerySearch.tsx` delimita o formulário e a interação; `Gallery.tsx` controla consulta, filtros e paginação. A normalização e o predicado SQLite existentes continuam compartilhados. `frontend-review.css` documenta os limites responsivos; o backend de importação e proteção não foi alterado.

## Dívidas identificadas

- CSS histórico ainda contém regras repetidas e `!important`; consolidar por componente gradualmente, com as mesmas verificações visuais.
- `App.tsx` e `Gallery.tsx` concentram muitas responsabilidades; extração inicial da busca feita nesta release, demais extrações precisam de tarefas próprias.
- Alguns fluxos secundários usam `prompt`/`confirm` nativos. Unificar os diálogos futuramente, preservando decisões de segurança.
- Os testes visuais por screenshot são evidências para inspeção; não substituem homologação prolongada no dispositivo do usuário.

A revisão cobre todas as seções listadas, mas não declara ausência universal de defeitos nem uma reescrita completa do frontend.
