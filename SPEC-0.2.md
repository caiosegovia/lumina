# Lumina 0.2 — especificação congelada

Este documento transforma a proposta aprovada em requisitos verificáveis. A release só pode ser chamada `0.2.0` quando todos os itens obrigatórios estiverem validados.

## Jobs e pipeline

- `JOB-01`: análise e consolidação executam em segundo plano sem bloquear navegação.
- `JOB-02`: SQLite é a fonte de verdade para trabalho, item, contador, etapa e erro.
- `JOB-03`: existe no máximo um trabalho de escrita ativo por biblioteca.
- `JOB-04`: filas e processos possuem limites explícitos de concorrência.
- `JOB-05`: descoberta, validação, metadados, hash, deduplicação, cópia, verificação, promoção, miniatura, backup e verificação são etapas persistentes e idempotentes.
- `JOB-06`: pausa não inicia nova operação e não abandona uma cópia pela metade.
- `JOB-07`: cancelamento remove somente temporários pertencentes ao trabalho.
- `JOB-08`: trabalhos interrompidos são oferecidos para retomar ou descartar após reinício.
- `JOB-09`: fonte e arquivos-mestre válidos nunca são removidos.

## Progresso

- `PRG-01`: snapshot contém etapa/arquivo, itens e bytes, percentuais geral/etapa, velocidade móvel, ETA e contadores.
- `PRG-02`: estado do acervo e backup são exibidos separadamente.
- `PRG-03`: métricas são calculadas no Rust, persistidas e sobrevivem à recarga.
- `PRG-04`: eventos aceleram a interface, mas não substituem o snapshot SQLite.

## Processos externos

- `PROC-01`: toda execução passa pelo único `ProcessRunner`; não há `Command::new` fora dele.
- `PROC-02`: Windows usa `CREATE_NO_WINDOW` e captura stdout/stderr.
- `PROC-03`: runner suporta timeout, cancelamento e limite de concorrência.
- `PROC-04`: eventos guardam ferramenta, comando lógico, duração, saída, código e erro sanitizado.
- `PROC-05`: dependência ausente produz orientação acionável e logs não contêm segredos.

## Validação e miniaturas

- `VAL-01`: imagens comuns são decodificadas; extensão isolada não comprova validade.
- `VAL-02`: vídeo usa FFprobe e decodificação de frame; HEIC usa FFmpeg quando necessário.
- `VAL-03`: RAW usa ExifTool e prévia embarcada quando disponível.
- `VAL-04`: estados cobrem válido, válido sem prévia, não suportado, corrompido, ilegível, timeout e dependência ausente.
- `VAL-05`: falha aponta etapa e coloca item em revisão sem alterar a fonte.
- `THM-01`: cache JPEG/WebP real respeita orientação e usa frame/prévia para vídeo/RAW.
- `THM-02`: chave inclui hash e versão do gerador; cache ausente/antigo é regenerado.
- `THM-03`: protocolo interno restrito serve somente miniaturas catalogadas.
- `THM-04`: cache pode ser limpo e reconstruído sem tocar originais.

## Atividade, diagnóstico e arquitetura

- `ACT-01`: atividade ao vivo filtra erros, duplicatas, ignorados e concluídos.
- `ACT-02`: recibo é persistente e exportável em JSONL e CSV.
- `ACT-03`: falhas recuperáveis podem ser tentadas novamente e possuem diagnóstico copiável.
- `ACT-04`: fonte desconectada é diferente de arquivo removido.
- `ARC-01`: módulos separam library, catalog, jobs, pipeline, media, storage, backup, process e events.
- `ARC-02`: migrations são versionadas; transações são curtas; processamento não segura conexão SQLite.
- `ARC-03`: temporários possuem job proprietário; destino é decidido e persistido uma vez.
- `ARC-04`: duas instâncias não escrevem simultaneamente na mesma biblioteca.

## Qualidade e release

- `TST-01`: testes unitários cobrem métricas, estados, cancelamento, caminhos, logs, erros, miniaturas e concorrência.
- `TST-02`: integração cobre pausa/cancelamento por etapa, reinício, idempotência, fonte offline, espaço, ferramentas ausentes e formatos inválidos.
- `TST-03`: interface cobre progresso, controle, recuperação, retry, navegação, recibo, miniaturas e acessibilidade básica.
- `TST-04`: carga cobre 100 mil ativos, lotes grandes, vídeos, memória, retomada e consultas.
- `REL-01`: nenhuma janela CMD aparece; fontes permanecem byte a byte intactas; cópia inválida nunca é promovida.
- `REL-02`: Rust, frontend, release, MSI, NSIS e portátil passam; artefatos possuem manifesto e SHA-256.
- `REL-03`: matriz, relatório automatizado, aceite manual e limitações autorizadas acompanham o pacote.

## Fora da 0.2

IA/faces, similaridade visual, Google Takeout, mapa completo, compartilhamento, servidor Linux, exclusão automática, editor e API direta do Google.
