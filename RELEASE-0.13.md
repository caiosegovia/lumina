# Lumina 0.13 — pacote de organização e confiança

Atualização de homologação 0.13.1: feedback explícito no reparo de miniaturas, navegação anterior/próxima no preview e estado vazio informativo na tela de duplicatas.

## Entrega consolidada

- recuperação de miniaturas automática, idempotente e sem modal de retomada;
- auditoria e reparo direcionado de miniaturas na área de Proteção;
- favoritos, avaliação de zero a cinco estrelas, descrição e fila “revisar depois”;
- ações em lote para organizar grandes seleções;
- filtros por favoritos, avaliação mínima e revisão pendente;
- visões salvas para reutilizar combinações de filtros;
- decisões persistentes para duplicatas: manter, revisar ou marcar candidatas;
- bloqueio duplo, na interface e no backend, para candidatas sem réplica verificada;
- exportação de diagnóstico sem nomes, caminhos, GPS ou hashes pessoais;
- trilha de alterações de data e estado pessoal no catálogo.

## Invariantes

- nenhuma decisão de duplicata remove arquivos;
- fontes originais não são alteradas pelo inventário;
- metadados pessoais ficam no catálogo, sem modificar o arquivo de mídia;
- miniaturas são cache descartável e podem ser regeneradas;
- operações de proteção continuam em segundo plano e são retomáveis.

## Homologação recomendada

1. Abrir uma biblioteca existente e confirmar que não aparece retomada de miniaturas.
2. Favoritar, avaliar, descrever e marcar fotos para revisão; reiniciar e confirmar persistência.
3. Salvar uma visão com filtros combinados e reaplicá-la.
4. Marcar decisões em grupos de duplicatas protegidos e não protegidos.
5. Auditar/reparar miniaturas em Proteção e navegar durante o processamento.
6. Exportar o diagnóstico e confirmar que o JSON contém apenas dados agregados.
