# Homologação — 0.29.1-beta.1

Status: candidata validada localmente para homologação. A aprovação no dispositivo do usuário permanece pendente. Tag de distribuição: `v0.29.1-beta.1`.

## Busca e galeria

1. Instalar sobre a versão anterior; conferir caminhos, tags, favoritos e álbuns.
2. Buscar pelo nome de um arquivo conhecido, usando botão e Enter. O campo não deve aumentar de altura.
3. Durante carregamento, enviar outra consulta: somente a última pode permanecer no resultado.
4. Buscar por tag, álbum, equipamento e lugar cadastrados.
5. Combinar busca com Fotos/Vídeos, ano e filtros avançados. Remover pills individualmente e usar Limpar tudo.
6. Buscar texto inexistente; limpar pela ação do estado vazio.
7. Testar Ctrl+F, Esc, repetir a mesma busca e carregar uma visão salva.
8. Abrir inspetor em grade/lista, redimensionar, navegar, testar zoom, vídeo e Explorer.

## Atividades

1. Importar pequena fonte de teste e acompanhar análise, consolidação e proteção.
2. Conferir Aguardando, Em execução, Atenção e Histórico.
3. Ao concluir, o job deve sair da execução e aparecer no histórico.
4. Atualizar atividades manualmente; conferir horário da consulta e mensagens de erro quando aplicáveis.
5. Conferir processamento de previews/metadados e avisos de atualização antiga sem conclusão automática de travamento.

## Revisão visual

Navegar por todas as nove seções, nos dois temas. Testar 1400×900, 1100×700, 1000×700 e escala Windows de 100%, 125%, 150%. Verificar botões legíveis, ausência de rolagem horizontal da página, filtros, lista, inspetor, diálogos e caminhos longos.

## Estabilidade

Executar buscas, filtros e navegação durante um job; manter uso por 20 minutos. Em caso de falha, registrar seção, sequência de ações, resolução/escala e exportar diagnóstico do aplicativo.

## Critérios de aceite

| Área | Resultado esperado |
| --- | --- |
| Busca | Botão e Enter retornam o mesmo conjunto; última consulta prevalece; sem expansão do campo |
| Filtros | Pills correspondem aos filtros reais; remoção individual e limpeza total atualizam os resultados |
| Paginação | Rolar uma busca com mais de 100 itens não zera a contagem nem mistura outra consulta |
| Janelas pequenas | Buscar, limpar e ordenar continuam clicáveis; barra não encobre ações; página sem rolagem horizontal |
| Visão salva | Texto da busca e filtros refletem a visão escolhida |
| Atividades | Aguardando não é confundido com execução; conclusão aparece no histórico; atualização manual responde |
| Regressão | Importação/proteção concluem; fontes preservadas; foto, vídeo, zoom, tags e favoritos continuam funcionando |

Registrar cada item como OK ou FALHOU, com passos para reproduzir e captura de tela. A release é candidata: aprovação local e CI não substituem o aceite do catálogo real no dispositivo de homologação.

## Evidências de desenvolvimento

- Frontend: 64 testes aprovados (10 arquivos).
- Rust: 150 testes aprovados; 2 ignorados. Formatação e Clippy (`-D warnings`) aprovados.
- TypeScript/Vite e empacotamento MSI/NSIS aprovados. `npm audit`: zero vulnerabilidades reportadas.
- Matriz de navegador: 1400×900, 1100×700, 1000×700, 800×560 e 667×467; temas claro/escuro, DPR 1/1,25/1,5 conforme cenário. As larguras menores simulam redução de área útil; isso não equivale a validar todas as configurações nativas de escala do Windows.
- Nove seções navegadas em cada cenário; verificações de geometria, consultas sobrepostas, filtros, estado vazio, limpeza, lista, inspetor e atalhos. Screenshots gerados em `artifacts/0.29.1/frontend`.
- Executável extraído do MSI final: catálogo isolado; 5 arquivos de origem resultando em 4 mídias únicas; análise, consolidação, deduplicação e réplica verificada; comparação/undo; índice de descoberta; zero falhas técnicas.
- No mesmo executável: 50 buscas por nomes reais do catálogo com botão/Enter, limpeza, preview de foto, reprodução de vídeo, zoom e tela cheia. Origem preservada e fechamento normal sem marcador de sessão pendente.
- Evidência desktop: `artifacts/0.29.1/desktop-1790772791001/result.json` e screenshots. O teste anterior tinha assumido um nome canônico fixo para uma duplicata; passou a usar o nome efetivamente catalogado, sem mudança no algoritmo de deduplicação.
- CI obrigatório inclui testes unitários, build, matriz de navegador, auditoria, formatação, Rust e ferramentas embarcadas. Consultar a execução vinculada ao PR #3 antes da publicação.

Os testes de navegador usam dados sintéticos; o smoke desktop usa arquivos reais gerados para teste, sem mocks de IPC. Instalação sobre catálogo existente, uso prolongado e escala nativa no dispositivo oficial fazem parte do aceite manual acima.
