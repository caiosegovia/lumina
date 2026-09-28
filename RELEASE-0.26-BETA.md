# Lumina 0.26.0-beta.1 — galeria, revisão e descoberta

Pacote corretivo sobre a 0.25.1, sem migração destrutiva de catálogo e sem alteração dos originais. A homologação do zoom da 0.25 continua registrada como reprovada; esta é uma nova candidata, não uma aprovação retroativa.

## Entregas

- Visualizador separado da galeria, com geometria compartilhada entre fotos e vídeos, zoom ancorado no cursor, pan limitado e atualizações agrupadas por quadro.
- Controles de zoom e reprodução fora da área transformada. Metadados possuem rolagem independente; tela cheia reserva espaço para navegação e controles. Em janelas menores, o inspetor usa painel flutuante sem comprimir a galeria.
- Carregamento HD adiado durante navegação rápida, descarte de respostas obsoletas e encerramento do vídeo ao trocar de mídia. “1:1” refere-se aos pixels da prévia limitada, não à decodificação do original dentro do WebView.
- Revisão → Falhas técnicas abre os arquivos afetados, com expansão dos motivos por etapa, caminhos, datas disponíveis e acesso ao Explorer. Paginação de 50 arquivos e regra compartilhada com o contador, sem contar duas vezes um arquivo que falhou em duas etapas.
- Descobrir renderiza quatro grupos por seção inicialmente, com expansão progressiva e até 12 prévias por grupo. Miniaturas são solicitadas perto da área visível. Cache de resultados por biblioteca por 30 segundos, com atualização explícita.
- Índice visual incremental v2: análise de prévias pequenas, decodificação de originais em subprocessos já limitados, memória de decodificação interna limitada e transações curtas por arquivo. Prévia temporária exclusiva é removida após análise; miniaturas existentes são reutilizadas.
- Progresso e cancelamento de análise visual e localização, exclusão mútua entre essas operações e preservação das gravações confirmadas. Sair do app também sinaliza cancelamento.
- Lugares preservam nomes manuais e informam a origem dos nomes. O fallback de cidade próxima passa a ser explicitamente aproximado. O geocodificador offline só é aceito com distância informada de até 25 km; isso continua sendo estimativa de localidade, não endereço confirmado.
- Resolução de lugares em lotes de 50 com gravações curtas. Viagens não unem saltos superiores a 250 km ou 72 horas; não oferecem o filtro incorreto que antes abria apenas o primeiro lugar da viagem.
- Plano de limpeza de duplicatas respeita decisões “manter” e “revisar”, exige decisão explícita de candidatura e réplica verificada. Continua somente simulação/exportação, sem excluir arquivos.

## Preservado

Setup com escolha de acervo e réplica, importação por jobs, proteção, seleção no Explorer, datas de captura, favoritos, tags, temas e tipografia. Não há reconhecimento de pessoas, verificador independente nem exclusão automática.

## Instalação e homologação

Use o instalador `.exe` da tag `v0.26.0-beta.1` (MSI alternativo). A versão numérica do instalador Tauri é `0.26.0-1`. Feche a versão anterior antes de atualizar e preserve uma cópia do catálogo antes de homologar.

Roteiro: [VALIDATION-0.26-BETA.md](VALIDATION-0.26-BETA.md). Decisões técnicas e limites: [ARCHITECTURE-0.26.md](ARCHITECTURE-0.26.md).

O índice visual v2 requer nova análise sob demanda; isso não reimporta nem duplica os arquivos. Recalcular lugares é explícito e mantém os nomes manuais. GPS sem nomes ricos continua sujeito aos limites da base offline.
