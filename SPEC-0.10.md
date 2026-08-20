# Lumina 0.10.0 — estabilização arquitetural

Esta versão elimina trabalho pesado do caminho da interface, torna as filas e a verificação recuperáveis, coordena I/O e processos externos, reduz escritas de progresso, remove Base64 das miniaturas e prepara catálogo e galeria para 100 mil mídias.

Regras de segurança mantidas:

- origens são somente leitura;
- cópias são promovidas apenas depois de SHA-256 confirmado;
- falha de réplica nunca resulta em estado protegido;
- manifesto e snapshot do catálogo são substituídos atomicamente;
- biblioteca, réplica e configuração real do usuário não são usadas nos testes automatizados.

O contrato completo e suas evidências estão em `ARCHITECTURE-STABILIZATION.md` e `TEST-REPORT-0.10.md`.
