# Lumina 0.10.0 — estabilização arquitetural

Esta versão elimina trabalho pesado do caminho da interface, torna as filas e a verificação recuperáveis, coordena I/O e processos externos, reduz escritas de progresso, remove Base64 das miniaturas e prepara catálogo e galeria para 100 mil mídias.

Regras de segurança mantidas:

- origens são somente leitura;
- cópias são promovidas apenas depois de SHA-256 confirmado;
- falha de réplica nunca resulta em estado protegido;
- manifesto e snapshot do catálogo são substituídos atomicamente;
- biblioteca, réplica e configuração real do usuário não são usadas nos testes automatizados.

O contrato completo e suas evidências estão em `ARCHITECTURE-STABILIZATION.md` e `TEST-REPORT-0.10.md`.

## Pendências observadas no teste produtivo

A 0.10.0 estabiliza a concorrência e evita que trabalhos de mídia sejam executados diretamente pela interface, mas o teste manual identificou dois pontos que permanecem no próximo ciclo:

- a abertura da Visão geral ainda apresenta latência perceptível; o próximo pacote deve medir separadamente consulta, montagem dos agregados, disponibilidade dos volumes e renderização, mantendo rollups pré-calculados e cache com invalidação por evento;
- os insights atuais são corretos, porém pouco orientados à decisão; devem evoluir para tendências, crescimento por período, maiores consumidores de espaço, cobertura de proteção por origem/ano, qualidade de metadados e recomendações priorizadas com impacto estimado.

Esses itens não anulam os testes arquiteturais da 0.10.0, mas ficam explicitamente registrados como critérios de produto e desempenho para a próxima versão.
