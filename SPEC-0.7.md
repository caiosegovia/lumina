# Lumina 0.7 — performance e transparência

## Entregas

- Validação estrutural de RAW incorporada à extração ExifTool em lote, eliminando processos externos repetidos por arquivo.
- Normalização de caminhos do Windows para garantir que metadados e validações em cache sejam realmente reutilizados.
- Cache incremental entre análises quando caminho, tamanho e data de modificação permanecem iguais; SHA-256 e metadados anteriores são reaproveitados sem reduzir a precisão da deduplicação.
- Progresso orientado por bytes, velocidade e tempo restante na Central de Atividade.
- Métricas persistentes separadas para metadados, validação, hash, cópia/verificação, miniaturas, réplica/verificação e totais.
- A mídia entra no catálogo após a cópia mestre ser verificada; miniatura e réplica são estados posteriores e visíveis.
- Alerta quando pasta-mestre e réplica usam a mesma unidade, explicando impacto em velocidade e proteção física.
- Paralelismo continua limitado por recurso para preservar a responsividade e evitar saturação de HDs externos.

## Integridade

A primeira análise de conteúdo continua calculando SHA-256 integral. A cópia mestre e a réplica continuam usando temporário, `fsync`, leitura de verificação e promoção atômica. Nenhuma mídia de origem é movida, editada ou excluída.
