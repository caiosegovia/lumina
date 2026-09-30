# Roteiro de homologação — 0.26.0-beta.1

Status: candidata de homologação. Os testes automatizados locais não aprovam o dispositivo de produção do usuário. Resultados locais consolidados ao final da execução abaixo.

## 1. Instalação e caminhos — regressão obrigatória

1. Fechar o Lumina anterior, instalar a 0.26 e abrir a biblioteca de homologação. Conferir acervo e réplica existentes.
2. Em perfil separado/instalação limpa, confirmar que o setup exige e permite configurar os dois caminhos. Não apontar uma biblioteca definitiva para pastas de teste.
3. Importar uma pasta pequena com foto, vídeo e uma cópia duplicada. Acompanhar análise e consolidação até o término. Proteger e aguardar a réplica verificada.
4. Confirmar que datas de captura, pastas por data, favoritos e tags anteriores continuam iguais.

## 2. Foto e vídeo — principal gate desta versão

1. Abrir uma foto grande, aguardar “Prévia HD”. Testar +, −, roda do mouse, arraste, Ajustar, 1:1 e Preencher.
2. O ponto sob o cursor deve orientar o zoom; pan não deve deixar a mídia perdida fora da área. Ajustar recupera o enquadramento.
3. Trocar 30 vezes entre fotos e vídeos, incluindo troca rápida. A prévia e os metadados devem corresponder ao arquivo atual.
4. Repetir com vídeo: reproduzir, pausar, buscar outro instante, alterar volume, ampliar e arrastar. Os controles devem continuar acessíveis fora da imagem ampliada.
5. Alternar tela cheia, lista/miniaturas e redimensionar a janela. Repetir com escala Windows 100%, 125% e 150%. Nome, fechar, navegação e zoom não podem desaparecer.
6. Em janela estreita, o inspetor flutua sobre a galeria. Em tela cheia estreita, os metadados ficam disponíveis ao sair da tela cheia; os controles de mídia têm prioridade.

Reprovar se houver congelamento, fechamento, controles inacessíveis ou percentual mudando sem ampliação real. Registrar arquivo/tipo, resolução da janela, DPI, ação e diagnóstico.

## 3. Revisão → Falhas técnicas

1. Clicar no card. Conferir total de arquivos e expandir dois itens: preview, metadados, motivo, caminho e data quando disponíveis.
2. Um arquivo com duas falhas aparece uma vez, com os dois motivos. Pendência sem erro não deve ser apresentada como falha técnica.
3. Se houver mais de 50 arquivos, navegar nas páginas. Atualizar a lista depois de uma reparação.
4. “Mostrar arquivo no Explorador” deve selecionar o arquivo existente. Arquivo ausente deve gerar mensagem, não abrir pasta aleatória.
5. Falha de formato não pode ser prometida como resolvida apenas por repetir a tentativa. Exportar diagnóstico se persistir.

## 4. Descobrir

1. Abrir e rolar: quatro grupos iniciais por seção, mais grupos sob demanda. Não disparar centenas de carregamentos fora da área visível.
2. Analisar biblioteca: conferir progresso, navegar para outra seção e voltar. Cancelar; aguardar a parada, executar novamente e confirmar reutilização do índice já salvo.
3. Recalcular lugares: observar progresso e possibilidade de cancelar. Nomes manuais devem continuar iguais.
4. Comparar fotos com nomes ricos embutidos, GPS sem nomes e sem GPS. Verificar indicação de nome manual, nome do arquivo, estimativa offline ou coordenadas aproximadas. Sem GPS não recebe lugar inventado.
5. Abrir um lugar na galeria e conferir filtro. Viagens não devem abrir uma seleção enganosa baseada só no primeiro lugar.
6. Comparar e fazer curadoria de um burst: melhor candidata favoritada, alternativas enviadas para revisão, nada excluído. Quando o grupo tiver mais de 12 arquivos, a ação vale para o grupo inteiro, não só para as prévias exibidas.
7. Reabrir Descobrir rapidamente para testar cache. Usar Atualizar descobertas para obter novo snapshot após importação.

## 5. Duplicatas e segurança

1. Importar duas cópias idênticas: conferir um asset e múltiplas ocorrências, sem segunda cópia desnecessária no acervo.
2. Gerar plano sem decisão de remoção: nenhuma ocorrência deve ser elegível por ausência de decisão.
3. Marcar grupo/ocorrência para manter ou revisar: deve bloquear candidatura, mesmo com réplica verificada.
4. Selecionar explicitamente candidatas com réplica verificada: plano mostra candidatura e espaço potencial. Exportar e conferir os motivos.
5. Nenhuma ação desse plano remove arquivos físicos nesta beta.

## 6. Estabilidade e feedback

Usar por 30–60 minutos com o acervo de aproximadamente 9.300 arquivos, alternando galeria, jobs e Descobrir. Exportar diagnóstico ao final; se houver travamento, informar horário local e última ação. A homologação prolongada no outro dispositivo permanece uma etapa real, não substituída por smoke local.

Formato do retorno: `item | aprovado/reprovado | ação/arquivo | horário local | print/diagnóstico`.

## Evidências locais

- Frontend: 54 testes aprovados em 9 arquivos (`npm.cmd test`).
- Backend: 144 testes aprovados, zero falhas, 2 testes especiais ignorados por padrão. A suíte inclui catálogo de 100 mil assets, análise de 2 mil arquivos, pipeline completo com deduplicação/proteção, GPS, falhas paginadas e índice incremental.
- `cargo clippy --all-targets -- -D warnings`: aprovado sem avisos.
- `npm.cmd run build`: aprovado.
- Browser Edge headless: foto JPEG sintética 4000×3000 e vídeo real gerado por FFmpeg; zoom medido, reprodução, roda, tela cheia, posição dos controles e ausência de erros JavaScript. Cenários 1400×900/DPR 1, 1100×700/DPR 1,25 e 1000×700/DPR 1,5; claro e escuro. DPR simulado não equivale à homologação do DPI nativo do Windows.
- Dois testes especiais não executados nesta rodada: benchmark específico de dashboard em 100k/500k e fixture RAW pessoal `LUMINA_REAL_RAW_FIXTURE`. Os testes regulares de RAW/HEIC e catálogo de 100k foram executados.
- A primeira suíte paralela encontrou timeout na preparação de um teste que compartilhava o limitador global; o teste recebeu limitador próprio. A suíte final foi executada com `--test-threads=1`.
- Neste ambiente, recópia de recursos Tauri no target de debug retornou acesso negado. Testes/clippy usaram `TAURI_CONFIG={"bundle":{"resources":[]}}` somente nesses processos; ferramentas são resolvidas do diretório de fontes nesses testes. O build de distribuição usa a configuração completa de recursos, sem esse override.

Smoke do executável distribuído: aprovado no `lumina.exe` extraído administrativamente do MSI 0.26.0-1, com recursos embarcados presentes. Setup real em perfil isolado, análise de 5 arquivos sintéticos, consolidação em 4 assets (deduplicação exata), réplica verificada, índice de 3 fotos, segunda análise sem reprocessamento, zero falhas técnicas, zoom/foto em tela cheia, original byte a byte preservado e encerramento limpo. Não houve instalação sobre o perfil normal nem acesso à biblioteca do usuário.

O smoke desktop foi repetido incluindo reprodução e zoom do vídeo pelo protocolo real do WebView2: aprovado, com encerramento limpo. Evidência local: `artifacts/0.26/desktop-1790434107870/result.json` e capturas do visualizador.

Instaladores NSIS (`.exe`) e MSI gerados com configuração completa. A pasta antiga de recursos de build foi preservada em `artifacts/0.26/build-resources-preserved-*` para permitir cópia limpa; nenhum arquivo de galeria foi removido. A validação automatizada final foi repetida com `scripts/verify-0.26.ps1 -SkipDebugResourceCopy` e diagnósticos isolados.
