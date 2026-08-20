# Lumina 0.9 — performance e visão geral

## Objetivo

Reduzir leituras redundantes na primeira importação e transformar a visão geral em um painel de decisão da biblioteca.

## Pipeline

- Inventário por caminho, tamanho e modificação continua imediato.
- SHA-256 antecipado é restrito a arquivos com candidato de mesmo tamanho no lote ou catálogo.
- Arquivos sem candidato recebem SHA-256 durante a escrita do temporário; o temporário é relido e validado antes da promoção atômica.
- A origem nunca é movida, editada ou excluída.
- Concorrência é conservadora para mídia grande ou removível e limitada a quatro workers nos demais casos.
- Metadados e validação estrutural são extraídos juntos em lotes de até 200 arquivos.
- Miniaturas são enfileiradas após a catalogação; a galeria pode consultar e gerar uma prévia sob demanda enquanto a fila progride.
- Progresso usa itens realmente concluídos, bytes, velocidade, ETA e o contador da etapa ativa.
- Métricas registram perfil de armazenamento, bytes antecipadamente hasheados, hashes adiados, cache, análise, cópia e miniaturas.

## Visão geral

- Big numbers: quantidade, espaço, cobertura de proteção e período.
- Composição por tipo com quantidade e bytes.
- Linha do tempo anual com quantidade e bytes.
- Espaço livre no acervo e backup.
- Proteção por estado, fontes e disponibilidade.
- Insights de proteção pendente, ocorrências adicionais, miniaturas, datas suspeitas e fontes offline.
- Benchmark da última execução.
- Agregados incrementais por tipo, ano e proteção, mantidos por triggers SQLite.

## Segurança

Deduplicação exata permanece baseada em SHA-256. Tamanho serve apenas para eliminar candidatos impossíveis; nunca declara dois arquivos iguais. Toda nova cópia é verificada antes de entrar no acervo.
