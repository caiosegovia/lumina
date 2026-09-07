# Especificação — Lumina 0.18.0-beta.1

## Objetivo

Entregar uma beta homologável para uso diário controlado, eliminando a degradação observada durante o enriquecimento de aproximadamente 9.300 mídias, tornando proteção e atividades verificáveis e ampliando a organização local sem alterar originais.

## Gate de estabilidade e diagnóstico

- Metadados técnicos são extraídos em lotes de no máximo 100 arquivos; o JSON de cada lote é liberado antes do seguinte.
- O WAL recebe checkpoints passivos entre lotes, mantendo leitores responsivos e progresso durável.
- A cada 15 segundos são registrados memória residente, CPU acumulada, idade do heartbeat da interface, job/etapa ativos e contadores da fila.
- Heartbeat acima de 15 segundos ou memória acima de 1,5 GB gera alerta diagnóstico.
- Ciclos de jobs e processos ExifTool/FFmpeg/FFprobe registram operação, duração e resultado.
- Encerramento incompleto é detectado na próxima abertura.
- Erros não tratados do frontend são registrados e isolados por uma tela de recuperação.
- O diagnóstico exportável inclui métricas, filas, catálogo e logs rotativos, sanitizando caminhos, nomes de mídia, coordenadas, hashes e segredos.

## Proteção e atividades

- `Proteger agora` executa o comando real na Atividade e na tela de Proteção.
- Se outro escritor estiver ativo, a proteção entra em fila durável e inicia quando o slot for liberado.
- Cópias só recebem estado protegido após verificação por SHA-256.
- Espaço insuficiente, destino indisponível, pausa, retomada e falha permanecem acionáveis.
- Atividade oferece exportação direta do diagnóstico completo.

## Organização avançada

- Pessoas são identidades locais e reversíveis, criadas deliberadamente pelo usuário e associadas em lote na galeria.
- Remover uma pessoa apaga suas associações do catálogo sem alterar as mídias.
- A busca da galeria encontra associações pelo nome da pessoa.
- Tags aceitam hierarquia com `>` ou `/`, mantendo o nó pai no catálogo.
- Álbuns inteligentes continuam baseados em filtros persistentes e são recalculados ao abrir.
- Lugares agrupam coordenadas existentes em regiões aproximadas; arquivos sem GPS nunca recebem localização inferida.
- Viagens são sugeridas apenas quando existem três ou mais registros com GPS em datas próximas.

## Descoberta e curadoria

- Lugares e viagens convivem com memórias, sequências e similaridade visual.
- Resultados abrem a mídia ou comparação existente.
- Similaridade não equivale a duplicidade e nenhuma sugestão autoriza exclusão.
- Duplicatas permanecem compactas, expansíveis e protegidas por decisões humanas.

## Invariantes

- Fontes e originais são somente leitura.
- Nenhuma exclusão, movimentação ou edição automática existe.
- Índices derivados e associações pessoais podem ser removidos sem perda do catálogo principal.
- Observabilidade possui rotação e não grava conteúdo das mídias.
- Nenhum reconhecimento facial ou semântico é simulado: esta beta entrega curadoria nominal local; agrupamento facial automático depende de um modelo local auditado em ciclo posterior.

## Critérios de saída

- Testes de frontend e backend, TypeScript/Vite, Rustfmt, Clippy, auditoria e smoke do pacote aprovados.
- Migração do catálogo v14 para v15 aprovada.
- Proteção iniciada pela Atividade comprovada por regressão automatizada.
- Cenário de 9.300 entradas comprova lote máximo de 100 e 93 lotes descartáveis.
- Instalador, MSI, portátil, hashes, documentação e roteiro publicados juntos.

