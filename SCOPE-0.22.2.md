# Escopo funcional preservado — Lumina 0.22.2

## Objetivo do produto

O Lumina é uma biblioteca desktop local para inventariar, consolidar, pesquisar, inspecionar, organizar, deduplicar e proteger fotos e vídeos sem alterar as fontes. O usuário deve distinguir com confiança o que existe, onde está, o que é duplicado, o que foi verificado e qual trabalho ainda está pendente.

## Jornadas que a estabilização deve preservar

| Jornada | Contrato funcional |
|---|---|
| Biblioteca | Criar acervo mestre e réplica em pastas separadas; impedir duas instâncias escritoras. |
| Fontes | Cadastrar, reencontrar, marcar offline e sincronizar fontes sem apagar ocorrências ausentes automaticamente. |
| Importação | Descobrir, validar, hashear, revisar, consolidar, pausar, cancelar e retomar. |
| Integridade | Promover somente temporário verificado; deduplicar por conteúdo; manter ocorrências de origem. |
| Galeria | Grade/lista virtualizada, cursor, agrupamentos, filtros, pills, seleção, teclado e estado de navegação. |
| Inspeção | Preview progressivo de foto/RAW, vídeo com busca, zoom, tela cheia, comparação e metadados sob demanda. |
| Curadoria | Favoritos, nota, revisar depois, descrição, tags, pessoas, álbuns, datas corrigidas e desfazer. |
| Descoberta | Linha do tempo, lugares offline, correções manuais, viagens, bursts e similares já disponíveis. |
| Duplicatas | Grupos compactos, ocorrências expansíveis, decisões e plano de limpeza sem exclusão automática. |
| Proteção | Fila para réplica local, hash verificado, estados verdadeiros, falta de espaço e nova verificação. |
| Atividade | Trabalho ativo, atenção, histórico, heartbeat, progresso, próxima ação e erros acionáveis. |
| Dashboard | Snapshot útil imediato, capacidade, composição, ritmo, equipamentos e insights controlados. |
| Diagnóstico | Métricas locais, export sanitizado e evidência suficiente para atribuir travamento. |

## Escopo da 0.22.2

A 0.22.2 pode alterar infraestrutura, estados internos, persistência de leases, protocolos e composição de módulos, desde que preserve as jornadas acima e as migrações do catálogo. Ela entrega resiliência, não novidades visuais/editoriais.

Incluído:

- limites de arquivo, memória, protocolos, processos, caches e filas;
- scheduler e lifecycle supervisionados;
- jobs duráveis e recuperáveis;
- atomicidade de configuração, réplica e migração;
- observabilidade privada e segurança do WebView;
- testes de escala, fault injection, endurance, instalação e upgrade.

Fora da 0.22.2:

- novas análises editoriais, IA, reconhecimento facial ou nuvem;
- mapa novo, compartilhamento, edição destrutiva ou exclusão automática;
- redesign visual amplo;
- suporte a outras plataformas;
- ampliação acima de 100 mil mídias ou 64 GiB por arquivo.

## Comportamento para limites

- Arquivo acima de 64 GiB aparece no inventário como “não suportado pelo limite desta versão”, sem tentativa automática de decode/hash/cópia.
- Pressão de recursos pausa derivados e mantém navegação, estado e possibilidade de cancelar.
- Recurso temporariamente indisponível produz espera explícita com motivo; falha permanente sai da fila ativa.
- O usuário nunca precisa “reparar” cache derivado para abrir a biblioteca; reconstrução é automática, limitada e adiada quando necessário.

## Não regressão

Favoritos, tags, álbuns, pessoas, descrições, datas corrigidas, lugares manuais, decisões de duplicatas, fontes e histórico não podem ser perdidos no upgrade. Caches, snapshots e insights podem ser invalidados e reconstruídos.
