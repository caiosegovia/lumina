# Especificação — Lumina 0.16.0-beta.1

## Objetivo

Lapidar a experiência operacional validada na 0.15, sem alterar as garantias de integridade: explicar o ciclo dos jobs, tornar a revisão de duplicatas progressiva e aplicar uma linguagem coerente às superfícies do produto.

## Atividade

- Um contrato central classifica estados em `active`, `attention` ou `history`.
- Estados pós-importação informam resultado e próximo passo; não simulam processamento.
- Progresso aparece somente enquanto existe trabalho executável.
- Atualização usa intervalo adaptativo: 1 segundo sob atividade e 5 segundos em repouso.
- Eventos são recarregados apenas quando id, estado ou atualização do job mudam.
- Histórico permanece recolhível e evidências técnicas não são apagadas.

## Duplicatas

- Lista compacta, fechada por padrão, com pills de cópias, espaço, proteção e decisão.
- Filtros: todas, pendentes, protegidas, revisar e elegíveis.
- Ordenação: maior espaço, mais cópias e nome.
- Detalhes, preview, caminhos e decisões são montados somente ao expandir.
- Exibição progressiva em blocos de 50 grupos.
- Nenhuma exclusão automática; candidatura continua bloqueada sem réplica verificada.

## Coerência e arquitetura

- Estados de job e seus textos deixam de ser duplicados nas telas.
- A implementação antiga de Central de trabalhos e utilitários mortos foi removida.
- A chave original do equipamento é preservada ao filtrar, mesmo quando o nome apresentado é normalizado.
- Visão Geral, Biblioteca, Revisão, Fontes, Álbuns/Tags, Proteção e Importação mantêm o sistema visual, estados e ações já homologados na 0.15.

## Critérios de saída

- Testes de frontend, backend, build, Clippy e auditoria aprovados.
- Smoke test do portátil confirma abertura, responsividade e encerramento limpo.
- MSI, instalador EXE, pacote portátil, hashes, documentação e roteiro produzidos juntos.
